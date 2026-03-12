#![deny(clippy::unwrap_used)]
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use memmap2::Mmap;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

static RE_STATE_ID: Lazy<Regex> = Lazy::new(|| Regex::new(r"id\s*=\s*(\d+)").unwrap());
static RE_STATE_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r#"name\s*=\s*"([^"]*)""#).unwrap());
static RE_STATE_OWNER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"owner\s*=\s*([A-Z0-9]{3})").unwrap());
static RE_STATE_CORE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"add_core_of\s*=\s*([A-Z0-9]{3})").unwrap());
static RE_STATE_CLAIM: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"add_claim_by\s*=\s*([A-Z0-9]{3})").unwrap());
static RE_COUNTRY_COLOR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?m)^([A-Z0-9]{3})\s*=\s*\{\s*color\s*=\s*(?:rgb)?\s*\{\s*(\d+)\s+(\d+)\s+(\d+)\s*\}",
    )
    .unwrap()
});

/// 地图上下文状态 (常驻内存)
#[allow(dead_code)]
pub struct MapContext {
    pub width: u32,
    pub height: u32,
    pub province_ids: Vec<u32>,
    pub definitions: HashMap<u32, ProvinceDefinition>,
    pub country_colors: HashMap<String, RGBColor>,
    pub state_owners: HashMap<u32, String>, // province_id -> owner_tag
    pub province_to_state: HashMap<u32, u32>, // province_id -> state_id
    pub state_to_provinces: HashMap<u32, Vec<u32>>, // state_id -> province_ids

    // 渲染查找表 (LUT) - 索引为 Province ID
    // 使用 Vec<[u8; 3]> 替代 HashMap 以获得 O(1) 访问速度
    pub province_color_lut: Vec<[u8; 3]>,
    pub state_color_lut: Vec<[u8; 3]>,
    pub country_color_lut: Vec<[u8; 3]>,
    pub terrain_color_lut: Vec<[u8; 3]>,

    // 缓存每个省份的包围盒，用于快速提取轮廓
    pub province_bounds: HashMap<u32, BoundingBox>,

    // 预计算的省份和州轮廓 (packed x | y << 16)
    pub province_outlines: HashMap<u32, Vec<u32>>,
    pub state_outlines: HashMap<u32, Vec<u32>>,
    pub country_same_color_border_points: Vec<u32>,
}

pub struct MapState(pub Mutex<Option<MapContext>>);

impl Default for MapState {
    fn default() -> Self {
        MapState(Mutex::new(None))
    }
}

#[inline]
fn log_map_perf(label: &str, started_at: Instant) {
    #[cfg(debug_assertions)]
    println!(
        "[map] {}: {:.2}ms",
        label,
        started_at.elapsed().as_secs_f64() * 1000.0
    );
}

/// 省份定义结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceDefinition {
    pub id: u32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    #[serde(rename = "type")]
    pub province_type: String,
    pub coastal: bool,
    pub terrain: String,
    pub continent: u32,
}

/// 区域范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
}

/// 颜色结构
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RGBColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// 扩展省份定义，包含位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceInstance {
    pub definition: ProvinceDefinition,
    pub bounding_box: Option<BoundingBox>,
    pub pixels_count: u32,
}

/// 边缘点集合 (优化：使用 packed u32 存储以减少内存占用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceEdge {
    pub from_id: u32,
    pub to_id: u32,
    pub points: Vec<u32>,
}

/// 地图位图数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceMapData {
    pub width: u32,
    pub height: u32,
    pub province_ids: Vec<u32>,
    pub instances: Vec<ProvinceInstance>,
    pub edges: Vec<ProvinceEdge>,
}

/// 地图配置结构 (来自 default.map)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultMap {
    pub definitions: String,
    pub provinces: String,
    pub adjacencies: String,
    pub continent: String,
    pub rivers: String,
    pub terrain_definition: Option<String>,
}

/// 带有编码检测的文件读取 (优化版本: 优先尝试 UTF-8)
fn read_file_with_encoding(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;

    // 优先尝试 UTF-8 (现代 MOD 常用)
    if let Ok(utf8_str) = String::from_utf8(bytes.clone()) {
        return Ok(utf8_str);
    }

    // 失败后尝试 Windows-1252 (HOI4 原版常用)
    let (decoded, _, had_errors) = encoding_rs::WINDOWS_1252.decode(&bytes);
    if !had_errors {
        return Ok(decoded.to_string());
    }

    // 最后才使用昂贵的检测器
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(&bytes, true);
    let detected_encoding = detector.guess(None, true);
    let (decoded, _, _) = detected_encoding.decode(&bytes);
    Ok(decoded.to_string())
}

/// 解析 definition.csv
pub fn parse_definition_csv(path: &Path) -> Result<Vec<ProvinceDefinition>, String> {
    let content = read_file_with_encoding(path)?;
    let mut provinces = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split(|c| c == ';' || c == ',').collect();
        if parts.len() >= 8 {
            let id = parts[0]
                .trim()
                .parse::<u32>()
                .map_err(|e| format!("ID parse error: {}", e))?;
            let r = parts[1]
                .trim()
                .parse::<u8>()
                .map_err(|e| format!("R parse error: {}", e))?;
            let g = parts[2]
                .trim()
                .parse::<u8>()
                .map_err(|e| format!("G parse error: {}", e))?;
            let b = parts[3]
                .trim()
                .parse::<u8>()
                .map_err(|e| format!("B parse error: {}", e))?;
            let province_type = parts[4].trim().to_string();
            let coastal = parts[5].trim().to_owned().to_lowercase() == "true";
            let terrain = parts[6].trim().to_string();
            let continent = parts[7].trim().parse::<u32>().unwrap_or(0);

            provinces.push(ProvinceDefinition {
                id,
                r,
                g,
                b,
                province_type,
                coastal,
                terrain,
                continent,
            });
        }
    }

    Ok(provinces)
}

/// 解析 default.map
pub fn parse_default_map(path: &Path) -> Result<DefaultMap, String> {
    let content = read_file_with_encoding(path)?;

    let mut definitions = "definition.csv".to_string();
    let mut provinces = "provinces.bmp".to_string();
    let mut adjacencies = "adjacencies.csv".to_string();
    let mut continent = "continent.txt".to_string();
    let mut rivers = "rivers.bmp".to_string();
    let mut terrain_definition = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_lowercase();
            let value = parts[1].trim().trim_matches('"').to_string();

            match key.as_str() {
                "definitions" => definitions = value,
                "provinces" => provinces = value,
                "adjacencies" => adjacencies = value,
                "continent" => continent = value,
                "rivers" => rivers = value,
                "terrain_definition" => terrain_definition = Some(value),
                _ => {}
            }
        }
    }

    Ok(DefaultMap {
        definitions,
        provinces,
        adjacencies,
        continent,
        rivers,
        terrain_definition,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapLoadResult<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[tauri::command]
pub fn load_map_definitions(path: String) -> MapLoadResult<Vec<ProvinceDefinition>> {
    match parse_definition_csv(Path::new(&path)) {
        Ok(provinces) => MapLoadResult {
            success: true,
            message: format!("成功加载 {} 个省份定义", provinces.len()),
            data: Some(provinces),
        },
        Err(e) => MapLoadResult {
            success: false,
            message: e,
            data: None,
        },
    }
}

/// 边缘检测算法 (极致优化版)
pub fn detect_edges(width: u32, height: u32, province_ids: &[u32]) -> Vec<ProvinceEdge> {
    // 并行计算每行的边缘关系，使用 Vec 暂存点以提高写入性能
    let edge_map: HashMap<(u32, u32), Vec<u32>> = (0..height)
        .into_par_iter()
        .fold(HashMap::new, |mut acc: HashMap<(u32, u32), Vec<u32>>, y| {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let current_id = province_ids[idx];

                if x + 1 < width {
                    let right_id = province_ids[idx + 1];
                    if current_id != right_id {
                        let key = if current_id < right_id {
                            (current_id, right_id)
                        } else {
                            (right_id, current_id)
                        };
                        let points = acc.entry(key).or_default();
                        points.push(x | (y << 16));
                        points.push((x + 1) | (y << 16));
                    }
                }

                if y + 1 < height {
                    let down_id = province_ids[idx + (width as usize)];
                    if current_id != down_id {
                        let key = if current_id < down_id {
                            (current_id, down_id)
                        } else {
                            (down_id, current_id)
                        };
                        let points = acc.entry(key).or_default();
                        points.push(x | (y << 16));
                        points.push(x | ((y + 1) << 16));
                    }
                }
            }
            acc
        })
        .reduce(HashMap::new, |mut a, b| {
            for (key, points) in b {
                a.entry(key).or_default().extend(points);
            }
            a
        });

    // 并行对每个边缘的点集进行去重
    edge_map
        .into_par_iter()
        .map(|((from_id, to_id), mut points)| {
            points.sort_unstable();
            points.dedup();
            ProvinceEdge {
                from_id,
                to_id,
                points,
            }
        })
        .collect()
}

/// 解析 provinces.bmp 并映射到省份 ID
pub fn parse_provinces_bmp(
    path: &Path,
    definitions: &[ProvinceDefinition],
) -> Result<ProvinceMapData, String> {
    // 2. Load Provinces BMP (Ultra-Fast Native BMP Parsing)
    let mut map_file = fs::File::open(path).map_err(|e| format!("无法打开位图文件: {}", e))?;

    use std::io::{Read, Seek, SeekFrom};
    let mut header = [0u8; 54];
    map_file
        .read_exact(&mut header)
        .map_err(|e| format!("读取 BMP 头部失败: {}", e))?;

    let pixel_offset = u32::from_le_bytes(header[10..14].try_into().unwrap_or([0; 4])) as u64;
    let width = i32::from_le_bytes(header[18..22].try_into().unwrap_or([0; 4])) as u32;
    let height = i32::from_le_bytes(header[22..26].try_into().unwrap_or([0; 4])) as u32;

    let row_size = ((width * 3 + 3) & !3) as usize;
    let mut raw_pixels = vec![0u8; row_size * height as usize];
    map_file
        .seek(SeekFrom::Start(pixel_offset))
        .map_err(|e| format!("Seek 失败: {}", e))?;
    map_file
        .read_exact(&mut raw_pixels)
        .map_err(|e| format!("读取像素数据失败: {}", e))?;

    // 2. 创建颜色到 ID 的映射表 (LUT 优化: 24-bit RGB -> ID)
    // 使用 16M 的 Vec 作为查找表，实现 O(1) 查找。约占用 64MB 内存。
    let mut color_lut = vec![0u32; 1 << 24];
    for def in definitions {
        let color_idx = ((def.r as usize) << 16) | ((def.g as usize) << 8) | (def.b as usize);
        color_lut[color_idx] = def.id;
    }

    // 3. 并行转换像素到 ID (优化：避免 flat_map 和临时向量)
    let mut province_ids = vec![0u32; (width * height) as usize];
    province_ids
        .par_chunks_mut(width as usize)
        .enumerate()
        .for_each(|(y_inv, row)| {
            let y = height - 1 - y_inv as u32;
            let row_start = y as usize * row_size;
            let row_data = &raw_pixels[row_start..row_start + (width * 3) as usize];
            for (x, chunk) in row_data.chunks_exact(3).enumerate() {
                let color_idx =
                    ((chunk[2] as usize) << 16) | ((chunk[1] as usize) << 8) | (chunk[0] as usize);
                row[x] = color_lut[color_idx];
            }
        });

    // 4. 计算每个省份的包围盒和像素计数 (优化：使用 Vec 替代 HashMap 减少开销)
    let max_id = definitions.iter().map(|d| d.id).max().unwrap_or(0);
    let stats = province_ids
        .par_iter()
        .enumerate()
        .fold(
            || vec![(u32::MAX, u32::MAX, 0u32, 0u32, 0u32); (max_id + 1) as usize],
            |mut local_stats, (idx, &id)| {
                if id > 0 && id <= max_id {
                    let x = (idx as u32) % width;
                    let y = (idx as u32) / width;
                    let s = &mut local_stats[id as usize];
                    s.0 = s.0.min(x);
                    s.1 = s.1.min(y);
                    s.2 = s.2.max(x);
                    s.3 = s.3.max(y);
                    s.4 += 1;
                }
                local_stats
            },
        )
        .reduce(
            || vec![(u32::MAX, u32::MAX, 0u32, 0u32, 0u32); (max_id + 1) as usize],
            |mut a, b| {
                for i in 0..a.len() {
                    if b[i].4 > 0 {
                        a[i].0 = a[i].0.min(b[i].0);
                        a[i].1 = a[i].1.min(b[i].1);
                        a[i].2 = a[i].2.max(b[i].2);
                        a[i].3 = a[i].3.max(b[i].3);
                        a[i].4 += b[i].4;
                    }
                }
                a
            },
        );

    // 5. 组装 ProvinceInstance
    let instances = definitions
        .iter()
        .map(|def| {
            let stat = if def.id <= max_id {
                Some(&stats[def.id as usize])
            } else {
                None
            };
            ProvinceInstance {
                definition: def.clone(),
                bounding_box: stat.filter(|s| s.4 > 0).map(|s| BoundingBox {
                    min_x: s.0,
                    min_y: s.1,
                    max_x: s.2,
                    max_y: s.3,
                }),
                pixels_count: stat.map(|s| s.4).unwrap_or(0),
            }
        })
        .collect();

    // 6. 边缘检测
    let edges = detect_edges(width, height, &province_ids);

    Ok(ProvinceMapData {
        width,
        height,
        province_ids,
        instances,
        edges,
    })
}

#[tauri::command]
pub fn load_default_map(path: String) -> MapLoadResult<DefaultMap> {
    match parse_default_map(Path::new(&path)) {
        Ok(config) => MapLoadResult {
            success: true,
            message: "成功加载地图配置".to_string(),
            data: Some(config),
        },
        Err(e) => MapLoadResult {
            success: false,
            message: e,
            data: None,
        },
    }
}

#[tauri::command]
pub fn load_provinces_bmp(
    path: String,
    definitions: Vec<ProvinceDefinition>,
) -> MapLoadResult<ProvinceMapData> {
    match parse_provinces_bmp(Path::new(&path), &definitions) {
        Ok(data) => MapLoadResult {
            success: true,
            message: format!("成功解析地图位图 ({}x{})", data.width, data.height),
            data: Some(data),
        },
        Err(e) => MapLoadResult {
            success: false,
            message: e,
            data: None,
        },
    }
}

#[tauri::command]
pub fn get_province_map_binary(
    path: String,
    definitions: Vec<ProvinceDefinition>,
) -> Result<Vec<u8>, String> {
    // 1. 加载位图 (优化版: 使用 Mmap)
    let file = fs::File::open(Path::new(&path)).map_err(|e| e.to_string())?;
    let mmap = unsafe { Mmap::map(&file).map_err(|e| e.to_string())? };

    if mmap.len() < 54 || &mmap[0..2] != b"BM" {
        return Err("无效的 BMP 文件".to_string());
    }

    let pixel_offset = u32::from_le_bytes(mmap[10..14].try_into().unwrap_or([0; 4])) as usize;
    let width = i32::from_le_bytes(mmap[18..22].try_into().unwrap_or([0; 4])) as u32;
    let height = i32::from_le_bytes(mmap[22..26].try_into().unwrap_or([0; 4])) as u32;
    let row_size = ((width * 3 + 3) & !3) as usize;

    // 2. 颜色查找表 (LUT)
    let mut color_lut = vec![0u32; 1 << 24];
    for def in definitions {
        let idx = ((def.r as usize) << 16) | ((def.g as usize) << 8) | (def.b as usize);
        color_lut[idx] = def.id;
    }

    // 3. 并行转换
    let mut province_ids = vec![0u32; (width * height) as usize];
    province_ids
        .par_chunks_mut(width as usize)
        .enumerate()
        .for_each(|(y_inv, row)| {
            let y = height - 1 - y_inv as u32;
            let row_start = pixel_offset + y as usize * row_size;
            let row_data = &mmap[row_start..row_start + (width * 3) as usize];
            for (x, chunk) in row_data.chunks_exact(3).enumerate() {
                let color_idx =
                    ((chunk[2] as usize) << 16) | ((chunk[1] as usize) << 8) | (chunk[0] as usize);
                row[x] = color_lut[color_idx];
            }
        });

    // 4. 直接返回字节数据 (零拷贝级转换)
    let byte_ptr = province_ids.as_ptr() as *const u8;
    let byte_len = province_ids.len() * 4;
    let bytes = unsafe { std::slice::from_raw_parts(byte_ptr, byte_len) };
    Ok(bytes.to_vec())
}

/// 根据提供的颜色映射生成着色地图数据 (RGBA)，支持下采样 (性能模式)
#[tauri::command]
pub fn generate_colored_map(
    province_ids: Vec<u32>,
    color_map: HashMap<u32, RGBColor>,
    default_color: RGBColor,
    width: u32,
    height: u32,
    downsample: Option<u32>,
) -> Vec<u8> {
    let scale = downsample.unwrap_or(1).max(1);

    if scale == 1 {
        // 原有逻辑：全分辨率渲染
        return province_ids
            .par_iter()
            .flat_map(|&id| {
                let color = color_map.get(&id).unwrap_or(&default_color);
                vec![color.r, color.g, color.b, color.a]
            })
            .collect();
    }

    // 性能模式：下采样渲染
    let new_width = width / scale;
    let new_height = height / scale;
    let mut pixels = Vec::with_capacity((new_width * new_height * 4) as usize);

    for y in 0..new_height {
        for x in 0..new_width {
            let orig_x = x * scale;
            let orig_y = y * scale;
            let idx = (orig_y * width + orig_x) as usize;

            if idx < province_ids.len() {
                let id = province_ids[idx];
                let color = color_map.get(&id).unwrap_or(&default_color);
                pixels.push(color.r);
                pixels.push(color.g);
                pixels.push(color.b);
                pixels.push(color.a);
            } else {
                pixels.extend_from_slice(&[
                    default_color.r,
                    default_color.g,
                    default_color.b,
                    default_color.a,
                ]);
            }
        }
    }

    pixels
}

/// 获取省份定义的原始颜色映射 (用于“省份”或“地形”图层)
#[tauri::command]
pub fn get_definition_color_map(definitions: Vec<ProvinceDefinition>) -> HashMap<u32, RGBColor> {
    let mut color_map = HashMap::with_capacity(definitions.len());
    for def in definitions {
        color_map.insert(
            def.id,
            RGBColor {
                r: def.r,
                g: def.g,
                b: def.b,
                a: 255,
            },
        );
    }
    color_map
}

/// 简单的国家颜色解析逻辑 (common/countries/colors.txt)
#[tauri::command]
pub fn load_country_colors(path: String) -> HashMap<String, RGBColor> {
    let mut colors = HashMap::new();
    let p = Path::new(&path);
    let content = read_file_with_encoding(p).unwrap_or_default();

    // 使用预编译的正则匹配 TAG = { color = { r g b } }
    for cap in RE_COUNTRY_COLOR.captures_iter(&content) {
        let tag = cap[1].to_string();
        let r = cap[2].parse().unwrap_or(0);
        let g = cap[3].parse().unwrap_or(0);
        let b = cap[4].parse().unwrap_or(0);
        colors.insert(tag, RGBColor { r, g, b, a: 255 });
    }

    colors
}

/// 根据州所有权和国家颜色生成省份颜色映射
#[tauri::command]
pub fn get_province_owner_color_map(
    states: Vec<StateDefinition>,
    country_colors: HashMap<String, RGBColor>,
) -> HashMap<u32, RGBColor> {
    let mut province_color_map = HashMap::new();
    for state in &states {
        if let Some(color) = country_colors.get(&state.owner) {
            for &province_id in &state.provinces {
                province_color_map.insert(province_id, *color);
            }
        }
    }
    province_color_map
}

/// 州定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDefinition {
    pub id: u32,
    pub name: String,
    pub provinces: Vec<u32>,
    pub owner: String,
    pub cores: Vec<String>,
    pub claims: Vec<String>,
}

/// 解析州文件 (history/states/*.txt)
pub fn parse_state_file(path: &Path) -> Result<StateDefinition, String> {
    let content = read_file_with_encoding(path)?;

    let mut id = 0;
    let mut name = String::new();
    let mut provinces = Vec::new();
    let mut owner = String::new();
    let mut cores = Vec::new();
    let mut claims = Vec::new();

    // 使用预编译的正则解析 (提升大量小文件解析速度)
    if let Some(caps) = RE_STATE_ID.captures(&content) {
        id = caps[1].parse().unwrap_or(0);
    }

    if let Some(caps) = RE_STATE_NAME.captures(&content) {
        name = caps[1].to_string();
    }

    if let Some(caps) = RE_STATE_OWNER.captures(&content) {
        owner = caps[1].to_string();
    }

    for cap in RE_STATE_CORE.captures_iter(&content) {
        cores.push(cap[1].to_string());
    }

    for cap in RE_STATE_CLAIM.captures_iter(&content) {
        claims.push(cap[1].to_string());
    }

    // 查找 provinces = { 1 2 3 }
    if let Some(start_idx) = content.find("provinces") {
        if let Some(open_brace) = content[start_idx..].find('{') {
            if let Some(close_brace) = content[start_idx + open_brace..].find('}') {
                let province_str =
                    &content[start_idx + open_brace + 1..start_idx + open_brace + close_brace];
                for p in province_str.split_whitespace() {
                    if let Ok(p_id) = p.parse::<u32>() {
                        provinces.push(p_id);
                    }
                }
            }
        }
    }

    Ok(StateDefinition {
        id,
        name,
        provinces,
        owner,
        cores,
        claims,
    })
}

/// 批量解析州目录 (并行版)
#[tauri::command]
pub fn load_all_states(states_dir: String) -> Vec<StateDefinition> {
    let started_at = Instant::now();
    let path = Path::new(&states_dir);
    if !path.exists() || !path.is_dir() {
        return Vec::new();
    }

    let entries: Vec<_> = fs::read_dir(path)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_else(|_| Vec::new());

    let states: Vec<_> = entries
        .par_iter()
        .filter(|p| p.is_file() && p.extension().map_or(false, |ext| ext == "txt"))
        .filter_map(|p| parse_state_file(p).ok())
        .collect();

    log_map_perf("rust.load_all_states", started_at);
    states
}

#[tauri::command]
pub fn initialize_map_context(
    state: tauri::State<MapState>,
    map_path: String,
    definitions_path: String,
    states_path: String,
    country_colors_path: String,
) -> Result<String, String> {
    let started_at = Instant::now();
    // 1. Load Definitions
    let definitions_started_at = Instant::now();
    let definitions_vec = parse_definition_csv(Path::new(&definitions_path))
        .map_err(|e| format!("无法加载省份定义文件 ({}): {}", definitions_path, e))?;
    let mut definitions = HashMap::with_capacity(definitions_vec.len());
    let mut color_to_id = HashMap::with_capacity(definitions_vec.len());
    for def in definitions_vec {
        color_to_id.insert((def.r, def.g, def.b), def.id);
        definitions.insert(def.id, def);
    }
    log_map_perf("rust.initialize_map_context.definitions", definitions_started_at);

    // 2. Load Provinces BMP (Ultra-Fast Native BMP Parsing with Mmap)
    let bmp_started_at = Instant::now();
    let map_file = fs::File::open(Path::new(&map_path))
        .map_err(|e| format!("无法打开地图位图 ({}): {}", map_path, e))?;
    let mmap = unsafe { Mmap::map(&map_file).map_err(|e| format!("内存映射失败: {}", e))? };

    if mmap.len() < 54 || &mmap[0..2] != b"BM" {
        return Err("不是有效的 BMP 文件".to_string());
    }

    let pixel_offset = u32::from_le_bytes(mmap[10..14].try_into().unwrap_or([0; 4])) as usize;
    let width = i32::from_le_bytes(mmap[18..22].try_into().unwrap_or([0; 4])) as u32;
    let height = i32::from_le_bytes(mmap[22..26].try_into().unwrap_or([0; 4])) as u32;
    let bpp = u16::from_le_bytes(mmap[28..30].try_into().unwrap_or([0; 2]));

    if bpp != 24 {
        return Err(format!("仅支持 24-bit BMP，当前为 {}-bit", bpp));
    }

    let row_size = ((width * 3 + 3) & !3) as usize; // BMP 行对齐

    // 创建颜色到 ID 的映射表 (LUT 优化: 24-bit RGB -> ID)
    let mut color_lut = vec![0u32; 1 << 24];
    for def in definitions.values() {
        let color_idx = ((def.r as usize) << 16) | ((def.g as usize) << 8) | (def.b as usize);
        color_lut[color_idx] = def.id;
    }

    // 并行转换像素到 ID，同时处理行倒序和对齐
    let mut province_ids = vec![0u32; (width * height) as usize];
    province_ids
        .par_chunks_mut(width as usize)
        .enumerate()
        .for_each(|(y_inv, row)| {
            let y = height - 1 - y_inv as u32;
            let row_start = pixel_offset + y as usize * row_size;
            let row_data = &mmap[row_start..row_start + (width * 3) as usize];
            for (x, chunk) in row_data.chunks_exact(3).enumerate() {
                let color_idx =
                    ((chunk[2] as usize) << 16) | ((chunk[1] as usize) << 8) | (chunk[0] as usize);
                row[x] = color_lut[color_idx];
            }
        });
    log_map_perf("rust.initialize_map_context.bmp", bmp_started_at);

    // 3. Load Country Colors
    let country_colors_started_at = Instant::now();
    let country_colors: HashMap<String, RGBColor> = load_country_colors(country_colors_path)
        .into_iter()
        .collect();
    log_map_perf(
        "rust.initialize_map_context.country_colors",
        country_colors_started_at,
    );

    // 4. Load States & Owners
    let states_started_at = Instant::now();
    let states = load_all_states(states_path);
    let mut state_owners = HashMap::with_capacity(definitions.len());
    let mut province_to_state = HashMap::with_capacity(definitions.len());
    let mut state_to_provinces = HashMap::with_capacity(states.len());

    // 并行生成州颜色 (优化)
    let province_to_state_color: HashMap<u32, [u8; 3]> = states
        .par_iter()
        .flat_map(|state| {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            use std::hash::{Hash, Hasher};
            state.id.hash(&mut hasher);
            let hash = hasher.finish();
            let r = ((hash & 0xFF) as u8 % 180) + 40;
            let g = (((hash >> 8) & 0xFF) as u8 % 180) + 40;
            let b = (((hash >> 16) & 0xFF) as u8 % 180) + 40;
            state
                .provinces
                .iter()
                .map(move |&p_id| (p_id, [r, g, b]))
                .collect::<Vec<_>>()
        })
        .collect();

    for state in &states {
        state_to_provinces.insert(state.id, state.provinces.clone());
        for &p_id in &state.provinces {
            state_owners.insert(p_id, state.owner.clone());
            province_to_state.insert(p_id, state.id);
        }
    }
    log_map_perf("rust.initialize_map_context.states", states_started_at);

    // 5. Generate Look-Up Tables (LUTs) for high-performance rendering
    let lut_started_at = Instant::now();
    let max_id = definitions.keys().max().copied().unwrap_or(0);
    let lut_size = (max_id + 1) as usize;

    // 并行生成各种渲染 LUT
    let (province_color_lut, (state_color_lut, (country_color_lut, terrain_color_lut))) =
        rayon::join(
            || {
                let mut lut = vec![[0, 0, 0]; lut_size];
                for def in definitions.values() {
                    if def.id < lut_size as u32 {
                        lut[def.id as usize] = [def.r, def.g, def.b];
                    }
                }
                lut
            },
            || {
                rayon::join(
                    || {
                        let mut lut = vec![[60, 60, 60]; lut_size];
                        for (&p_id, &color) in &province_to_state_color {
                            if p_id < lut_size as u32 {
                                lut[p_id as usize] = color;
                            }
                        }
                        lut
                    },
                    || {
                        rayon::join(
                            || {
                                let mut lut = vec![[40, 40, 40]; lut_size];
                                for (&p_id, owner) in &state_owners {
                                    if p_id < lut_size as u32 {
                                        if let Some(c) = country_colors.get(owner) {
                                            lut[p_id as usize] = [c.r, c.g, c.b];
                                        } else {
                                            lut[p_id as usize] = [128, 128, 128];
                                        }
                                    }
                                }
                                lut
                            },
                            || {
                                let mut lut = vec![[100, 100, 100]; lut_size];
                                for def in definitions.values() {
                                    if def.id < lut_size as u32 {
                                        lut[def.id as usize] = match def.terrain.as_str() {
                                            "plains" => [247, 166, 86],
                                            "forest" => [85, 139, 47],
                                            "hills" => [255, 215, 0],
                                            "mountain" => [139, 69, 19],
                                            "urban" => [128, 128, 128],
                                            "jungle" => [34, 139, 34],
                                            "marsh" => [47, 79, 79],
                                            "desert" => [244, 164, 96],
                                            "water" | "ocean" => [65, 105, 225],
                                            "lakes" => [65, 155, 225],
                                            _ => [200, 200, 200],
                                        };
                                    }
                                }
                                lut
                            },
                        )
                    },
                )
            },
        );
    log_map_perf("rust.initialize_map_context.lut", lut_started_at);

    // Calculate province bounds (Parallel optimization using Vec instead of HashMap)
    let bounds_started_at = Instant::now();
    let stats = province_ids
        .par_iter()
        .enumerate()
        .fold(
            || vec![(u32::MAX, u32::MAX, 0u32, 0u32, 0u32); lut_size],
            |mut local_stats, (idx, &id)| {
                if id > 0 && id <= max_id {
                    let x = (idx as u32) % width;
                    let y = (idx as u32) / width;
                    let s = &mut local_stats[id as usize];
                    s.0 = s.0.min(x);
                    s.1 = s.1.min(y);
                    s.2 = s.2.max(x);
                    s.3 = s.3.max(y);
                    s.4 += 1;
                }
                local_stats
            },
        )
        .reduce(
            || vec![(u32::MAX, u32::MAX, 0u32, 0u32, 0u32); lut_size],
            |mut a, b| {
                for i in 0..a.len() {
                    if b[i].4 > 0 {
                        a[i].0 = a[i].0.min(b[i].0);
                        a[i].1 = a[i].1.min(b[i].1);
                        a[i].2 = a[i].2.max(b[i].2);
                        a[i].3 = a[i].3.max(b[i].3);
                        a[i].4 += b[i].4;
                    }
                }
                a
            },
        );

    let mut province_bounds = HashMap::with_capacity(lut_size);
    for (id, s) in stats.into_iter().enumerate() {
        if s.4 > 0 {
            province_bounds.insert(
                id as u32,
                BoundingBox {
                    min_x: s.0,
                    min_y: s.1,
                    max_x: s.2,
                    max_y: s.3,
                },
            );
        }
    }
    log_map_perf("rust.initialize_map_context.bounds", bounds_started_at);

    // 6. 预计算省份和州轮廓 (极致性能优化)
    let outlines_started_at = Instant::now();
    let all_edges = detect_edges(width, height, &province_ids);
    let mut province_outlines: HashMap<u32, Vec<u32>> = HashMap::with_capacity(lut_size);
    let mut state_outlines: HashMap<u32, Vec<u32>> = HashMap::with_capacity(states.len());
    let mut country_same_color_border_points = Vec::new();

    for edge in all_edges {
        let packed_points = edge.points;

        if edge.from_id != 0 {
            province_outlines
                .entry(edge.from_id)
                .or_default()
                .extend(&packed_points);
        }
        if edge.to_id != 0 {
            province_outlines
                .entry(edge.to_id)
                .or_default()
                .extend(&packed_points);
        }

        // 处理州轮廓：如果边缘连接两个不同的州，则它是州界
        let from_state = province_to_state.get(&edge.from_id).copied();
        let to_state = province_to_state.get(&edge.to_id).copied();

        if from_state != to_state {
            if let Some(fs) = from_state {
                state_outlines.entry(fs).or_default().extend(&packed_points);
            }
            if let Some(ts) = to_state {
                state_outlines.entry(ts).or_default().extend(&packed_points);
            }
        }

        if edge.from_id != 0 && edge.to_id != 0 {
            let from_owner = state_owners.get(&edge.from_id);
            let to_owner = state_owners.get(&edge.to_id);
            let from_color = country_color_lut.get(edge.from_id as usize);
            let to_color = country_color_lut.get(edge.to_id as usize);

            if let (Some(from_owner), Some(to_owner), Some(from_color), Some(to_color)) =
                (from_owner, to_owner, from_color, to_color)
            {
                if from_owner != to_owner && from_color == to_color {
                    country_same_color_border_points.extend(&packed_points);
                }
            }
        }
    }

    // 并行去重轮廓点
    province_outlines.par_iter_mut().for_each(|(_, points)| {
        points.sort_unstable();
        points.dedup();
    });
    state_outlines.par_iter_mut().for_each(|(_, points)| {
        points.sort_unstable();
        points.dedup();
    });
    country_same_color_border_points.sort_unstable();
    country_same_color_border_points.dedup();
    log_map_perf("rust.initialize_map_context.outlines", outlines_started_at);

    // 7. Store in State
    let mut lock = state.0.lock().map_err(|_| "Failed to lock state")?;
    *lock = Some(MapContext {
        width,
        height,
        province_ids,
        definitions,
        country_colors,
        state_owners,
        province_to_state,
        state_to_provinces,
        province_color_lut,
        state_color_lut,
        country_color_lut,
        terrain_color_lut,
        province_bounds,
        province_outlines,
        state_outlines,
        country_same_color_border_points,
    });

    log_map_perf("rust.initialize_map_context.total", started_at);
    Ok(format!("Map initialized: {}x{}", width, height))
}

#[tauri::command]
pub fn get_province_outline(
    state: tauri::State<MapState>,
    province_id: u32,
) -> Result<Vec<u8>, String> {
    let context_guard = state.0.lock().map_err(|_| "Failed to lock map state")?;
    let context = context_guard
        .as_ref()
        .ok_or("Map context not initialized")?;

    if let Some(points) = context.province_outlines.get(&province_id) {
        // 直接返回原始内存字节数据，前端将其视为 Uint32Array
        let byte_ptr = points.as_ptr() as *const u8;
        let byte_len = points.len() * 4;
        let bytes = unsafe { std::slice::from_raw_parts(byte_ptr, byte_len) };
        Ok(bytes.to_vec())
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
pub fn get_state_outline(state: tauri::State<MapState>, state_id: u32) -> Result<Vec<u8>, String> {
    let context_guard = state.0.lock().map_err(|_| "Failed to lock map state")?;
    let context = context_guard
        .as_ref()
        .ok_or("Map context not initialized")?;

    if let Some(points) = context.state_outlines.get(&state_id) {
        let byte_ptr = points.as_ptr() as *const u8;
        let byte_len = points.len() * 4;
        let bytes = unsafe { std::slice::from_raw_parts(byte_ptr, byte_len) };
        Ok(bytes.to_vec())
    } else {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMetadata {
    pub width: u32,
    pub height: u32,
    pub province_count: usize,
}

fn darken_pixel(pixel: &mut [u8]) {
    pixel[0] = ((pixel[0] as u16 * 3) / 5) as u8;
    pixel[1] = ((pixel[1] as u16 * 3) / 5) as u8;
    pixel[2] = ((pixel[2] as u16 * 3) / 5) as u8;
}

fn apply_country_same_color_borders_to_preview(
    pixels: &mut [u8],
    target_width: u32,
    target_height: u32,
    map_width: u32,
    map_height: u32,
    border_points: &[u32],
) {
    if border_points.is_empty() || target_width == 0 || target_height == 0 {
        return;
    }

    let mut border_mask = vec![false; (target_width * target_height) as usize];

    for &packed in border_points {
        let src_x = packed & 0xFFFF;
        let src_y = packed >> 16;

        if src_x >= map_width || src_y >= map_height {
            continue;
        }

        let out_x = ((src_x as u64 * target_width as u64) / map_width as u64)
            .min((target_width - 1) as u64) as u32;
        let out_y = ((src_y as u64 * target_height as u64) / map_height as u64)
            .min((target_height - 1) as u64) as u32;

        border_mask[(out_y * target_width + out_x) as usize] = true;
    }

    for (index, is_border) in border_mask.into_iter().enumerate() {
        if !is_border {
            continue;
        }

        let pixel_index = index * 4;
        darken_pixel(&mut pixels[pixel_index..pixel_index + 4]);
    }
}

fn apply_country_same_color_borders_to_tile(
    pixels: &mut [u8],
    tile_size: u32,
    src_x_start: u32,
    src_y_start: u32,
    scale: u32,
    border_points: &[u32],
) {
    if border_points.is_empty() || tile_size == 0 || scale == 0 {
        return;
    }

    let src_x_end = src_x_start as u64 + tile_size as u64 * scale as u64;
    let src_y_end = src_y_start as u64 + tile_size as u64 * scale as u64;
    let mut border_mask = vec![false; (tile_size * tile_size) as usize];

    for &packed in border_points {
        let src_x = (packed & 0xFFFF) as u64;
        let src_y = (packed >> 16) as u64;

        if src_x < src_x_start as u64
            || src_x >= src_x_end
            || src_y < src_y_start as u64
            || src_y >= src_y_end
        {
            continue;
        }

        let out_x = ((src_x - src_x_start as u64) / scale as u64) as u32;
        let out_y = ((src_y - src_y_start as u64) / scale as u64) as u32;

        if out_x >= tile_size || out_y >= tile_size {
            continue;
        }

        border_mask[(out_y * tile_size + out_x) as usize] = true;
    }

    for (index, is_border) in border_mask.into_iter().enumerate() {
        if !is_border {
            continue;
        }

        let pixel_index = index * 4;
        darken_pixel(&mut pixels[pixel_index..pixel_index + 4]);
    }
}

#[tauri::command]
pub fn get_map_metadata(state: tauri::State<MapState>) -> Result<MapMetadata, String> {
    let lock = state.0.lock().map_err(|_| "Failed to lock state")?;
    let ctx = lock.as_ref().ok_or("Map not initialized")?;

    Ok(MapMetadata {
        width: ctx.width,
        height: ctx.height,
        province_count: ctx.province_ids.len(),
    })
}

#[tauri::command]
pub fn get_map_preview(
    state: tauri::State<MapState>,
    target_width: u32,
    target_height: u32,
    mode: String,
) -> Result<Vec<u8>, String> {
    let started_at = Instant::now();
    let lock = state.0.lock().map_err(|_| "Failed to lock state")?;
    let ctx = lock.as_ref().ok_or("Map not initialized")?;

    let map_width = ctx.width;
    let map_height = ctx.height;

    let scale_x = map_width as f32 / target_width as f32;
    let scale_y = map_height as f32 / target_height as f32;

    let mut pixels = vec![0u8; (target_width * target_height * 4) as usize];

    // Select LUT based on mode
    let lut = match mode.as_str() {
        "province" => &ctx.province_color_lut,
        "state" => &ctx.state_color_lut,
        "country" => &ctx.country_color_lut,
        "terrain" => &ctx.terrain_color_lut,
        _ => &ctx.province_color_lut, // Default fallback
    };

    // Parallel rendering for preview
    pixels
        .par_chunks_exact_mut(4)
        .enumerate()
        .for_each(|(i, pixel)| {
            let x = i as u32 % target_width;
            let y = i as u32 / target_width;

            let src_x = (x as f32 * scale_x) as u32;
            let src_y = (y as f32 * scale_y) as u32;

            if src_x < map_width && src_y < map_height {
                let idx = (src_y * map_width + src_x) as usize;
                if idx < ctx.province_ids.len() {
                    let pid = ctx.province_ids[idx] as usize;

                    if pid < lut.len() {
                        let c = lut[pid];
                        pixel[0] = c[0];
                        pixel[1] = c[1];
                        pixel[2] = c[2];
                        pixel[3] = 255; // Alpha
                    } else {
                        // Invalid ID?
                        pixel[0] = 0;
                        pixel[1] = 0;
                        pixel[2] = 0;
                        pixel[3] = 255;
                    }
                }
            }
        });

    if mode == "country" {
        apply_country_same_color_borders_to_preview(
            &mut pixels,
            target_width,
            target_height,
            map_width,
            map_height,
            &ctx.country_same_color_border_points,
        );
    }

    log_map_perf("rust.get_map_preview", started_at);
    Ok(pixels)
}

#[tauri::command]
pub fn get_province_at_point(
    state: tauri::State<MapState>,
    x: u32,
    y: u32,
) -> Result<Option<u32>, String> {
    let started_at = Instant::now();
    let lock = state.0.lock().map_err(|_| "Failed to lock state")?;
    let ctx = lock.as_ref().ok_or("Map not initialized")?;

    if x >= ctx.width || y >= ctx.height {
        return Ok(None);
    }

    let idx = (y * ctx.width + x) as usize;
    if idx < ctx.province_ids.len() {
        let result = Some(ctx.province_ids[idx]);
        log_map_perf("rust.get_province_at_point", started_at);
        Ok(result)
    } else {
        log_map_perf("rust.get_province_at_point", started_at);
        Ok(None)
    }
}

#[tauri::command]
pub fn get_map_tile_direct(
    state: tauri::State<MapState>,
    x: u32,
    y: u32,
    zoom: u32,
    mode: String,
) -> Result<Vec<u8>, String> {
    let started_at = Instant::now();
    let lock = state.0.lock().map_err(|_| "Failed to lock state")?;
    let ctx = lock.as_ref().ok_or("Map not initialized")?;

    let tile_size = 512;
    let scale = zoom.max(1);
    let mut pixels = vec![0u8; (tile_size * tile_size * 4) as usize];

    let map_width = ctx.width;
    let map_height = ctx.height;

    let src_x_start = x * tile_size * scale;
    let src_y_start = y * tile_size * scale;

    // Select LUT based on mode
    let lut = match mode.as_str() {
        "province" => &ctx.province_color_lut,
        "state" => &ctx.state_color_lut,
        "country" => &ctx.country_color_lut,
        "terrain" => &ctx.terrain_color_lut,
        _ => &ctx.province_color_lut, // Default fallback
    };

    // Using Rayon for parallel processing of rows within the tile if zoom is large?
    // Actually, for 512x512, single thread is usually fast enough if logic is simple.
    // But let's keep it simple sequential for now to avoid overhead, as simple array lookup is extremely fast.
    // However, if we want extreme speed, we can use par_chunks_mut for the output buffer.

    // Let's use parallel iterator for the rows to maximize speed
    pixels
        .par_chunks_exact_mut(tile_size as usize * 4)
        .enumerate()
        .for_each(|(ty_idx, row_pixels)| {
            let ty = ty_idx as u32;
            let src_y = src_y_start + ty * scale;

            if src_y >= map_height {
                return; // Leave as transparent/black
            }

            let row_start_idx = (src_y * map_width) as usize;

            for tx in 0..tile_size {
                let src_x = src_x_start + tx * scale;

                if src_x < map_width {
                    let idx = row_start_idx + src_x as usize;

                    // Safety check for bounds
                    if idx < ctx.province_ids.len() {
                        let pid = ctx.province_ids[idx] as usize;

                        if pid < lut.len() {
                            let c = lut[pid];
                            let p_idx = (tx * 4) as usize;
                            row_pixels[p_idx] = c[0];
                            row_pixels[p_idx + 1] = c[1];
                            row_pixels[p_idx + 2] = c[2];
                            row_pixels[p_idx + 3] = 255;
                        }
                    }
                }
            }
        });

    if mode == "country" {
        apply_country_same_color_borders_to_tile(
            &mut pixels,
            tile_size,
            src_x_start,
            src_y_start,
            scale,
            &ctx.country_same_color_border_points,
        );
    }

    log_map_perf("rust.get_map_tile_direct", started_at);
    Ok(pixels)
}
