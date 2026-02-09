use std::collections::HashMap;
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;
use m3u8_rs::playlist::Playlist;

const FULL_KEY: &str = "bcd151073c03b352e1ef2fd66c32209da9ca0afa";
const COORDI_NAMES: [(&str, f32, f32); 47] = [("JP1", 43.064615, 141.346807), ("JP2", 40.824308, 140.739998), ("JP3", 39.703619, 141.152684), ("JP4", 38.268837, 140.8721), ("JP5", 39.718614, 140.102364), ("JP6", 38.240436, 140.363633), ("JP7", 37.750299, 140.467551), ("JP8", 36.341811, 140.446793), ("JP9", 36.565725, 139.883565), ("JP10", 36.390668, 139.060406), ("JP11", 35.856999, 139.648849), ("JP12", 35.605057, 140.123306), ("JP13", 35.689488, 139.691706), ("JP14", 35.447507, 139.642345), ("JP15", 37.902552, 139.023095), ("JP16", 36.695291, 137.211338), ("JP17", 36.594682, 136.625573), ("JP18", 36.065178, 136.221527), ("JP19", 35.664158, 138.568449), ("JP20", 36.651299, 138.180956), ("JP21", 35.391227, 136.722291), ("JP22", 34.97712, 138.383084), ("JP23", 35.180188, 136.906565), ("JP24", 34.730283, 136.508588), ("JP25", 35.004531, 135.86859), ("JP26", 35.021247, 135.755597), ("JP27", 34.686297, 135.519661), ("JP28", 34.691269, 135.183071), ("JP29", 34.685334, 135.832742), ("JP30", 34.225987, 135.167509), ("JP31", 35.503891, 134.237736), ("JP32", 35.472295, 133.0505), ("JP33", 34.661751, 133.934406), ("JP34", 34.39656, 132.459622), ("JP35", 34.185956, 131.470649), ("JP36", 34.065718, 134.55936), ("JP37", 34.340149, 134.043444), ("JP38", 33.841624, 132.765681), ("JP39", 33.559706, 133.531079), ("JP40", 33.606576, 130.418297), ("JP41", 33.249442, 130.299794), ("JP42", 32.744839, 129.873756), ("JP43", 32.789827, 130.741667), ("JP44", 33.238172, 131.612619), ("JP45", 31.911096, 131.423893), ("JP46", 31.560146, 130.557978), ("JP47", 26.2124, 127.680932)];

pub fn get_live_stream_info(station_id: &String, timefree: bool, areafree: bool) -> Option<(String, String)> {
    if is_available_station_id(station_id) {
        if let Some(auth_token) = get_auth_token_4_station_id(station_id) {
            if let Ok(base_urls) = get_stream_base_urls(station_id, timefree, areafree) {
                return Some((auth_token, format!("{}?station_id={}&l=15&lsid=&type=b", base_urls[0], station_id)));
            }
        }
    }
    None
}

pub fn get_stations() -> eyre::Result<Vec<HashMap<String, String>>> {
    let response = reqwest::blocking::get("https://radiko.jp/v3/station/region/full.xml")?.text()?;
    Ok(to_stations_map(&response))
}

pub fn _get_playlists(url: &String, auth_token: &String) -> eyre::Result<Vec<String>> {
    let client = reqwest::blocking::Client::new();
    let response1 = client.get(url)
        .header("X-Radiko-AuthToken", auth_token)
        .send()?;
    if response1.status().is_success() {
        let contents = response1.text()?;
        let bytes: Vec<u8> = contents.as_bytes().to_vec();
        let _parsed = m3u8_rs::parse_playlist_res(&bytes);
        if let Ok(Playlist::MasterPlaylist(master)) = m3u8_rs::parse_playlist_res(&bytes) {
            let response2 = client.get(&master.variants[0].uri)
                .header("X-Radiko-AuthToken", auth_token)
                .send()?;
            if response2.status().is_success() {
                let contents = response2.text()?;
                let bytes: Vec<u8> = contents.as_bytes().to_vec();
                let _parsed = m3u8_rs::parse_playlist_res(&bytes);
                if let Ok(Playlist::MediaPlaylist(media)) = m3u8_rs::parse_playlist_res(&bytes) {
                    let mut urls = Vec::new();
                    for seg in media.segments {
                        urls.push(seg.uri);
                    }
                    return Ok(urls);
                }
            }
        }
    }
    Err(eyre::eyre!("Cannot get playlists !"))
}

fn to_stations_map(contents: &String) -> Vec<HashMap<String, String>> {
    let mut stations: Vec<HashMap<String, String>> = Vec::new();
    let mut reader = Reader::from_str(&contents);
    let mut buf = Vec::new();
    let mut station_map: Option<HashMap<String, String>> = None;
    let mut current_tag: Option<String> = None;

    reader.trim_text(true);
    loop {
        match reader.read_event(&mut buf) {

            Ok(Event::Start(ref e)) => {
                if e.name() == b"station" {
                    station_map = Some(HashMap::new());
                } 
                current_tag = Some(String::from_utf8(e.name().to_vec()).unwrap());
            },
            Ok(Event::End(ref e)) => {
                if e.name() == b"station" {
                    if let Some(station_info) = station_map {
                        stations.push(station_info);
                    }
                    station_map = None;
                }
                current_tag = None;
            },
            Ok(Event::Text(ref e)) => {
                if current_tag.is_some() && station_map.is_some() {
                    let tag = current_tag.unwrap();
                    let mut map = station_map.unwrap();
                    map.insert(tag.clone(), e.unescape_and_decode(&reader).unwrap());
                    station_map = Some(map);
                    current_tag = Some(tag);
                }
            },
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }

    stations
}

pub fn _get_station(station_id: &String) -> Option<HashMap<String, String>> {
    if let Ok(stations) = get_stations() {
        for map in stations {
            if let Some(id) = map.get("id") {
                if id == station_id {
                    return Some(map);
                }
            }
        }
    }
    None
}

pub fn is_available_station_id(station_id: &String) -> bool {
    if let Some(ids) = get_station_ids() {
        return ids.contains(station_id);
    }
    false
}

fn get_station_ids() -> Option<Vec<String>> {
    if let Ok(stations) = get_stations() {
        let mut ids: Vec<String> = Vec::new();
        for map in stations {
            if let Some(id) = map.get("id") {
                ids.push(id.clone());
            }
        }
        return Some(ids);
    }
    None
}

fn get_stream_base_urls(station_id: &String, timefree: bool, areafree: bool) -> eyre::Result<Vec<String>> {
    let response = reqwest::blocking::get(format!("https://radiko.jp/v3/station/stream/aSmartPhone7o/{}.xml", station_id))?.text()?;
    Ok(to_urls_map(&response, timefree, areafree))
}

fn to_urls_map(contents: &String, timefree: bool, areafree: bool) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    let mut reader = Reader::from_str(&contents);
    let mut buf = Vec::new();
    let mut url: bool = false;
    let mut pst: bool = false;

    reader.trim_text(true);
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name() == b"url" {
                    let mut tf = false;
                    if check_attr(&reader, e.attributes(), "timefree", "1") {
                        tf = true;
                    }
                    let mut af = false;
                    if check_attr(&reader, e.attributes(), "areafree", "1") {
                        af = true;
                    }
                    if tf == timefree && af == areafree {
                        url = true;
                    }
                } else if e.name() == b"playlist_create_url" {
                    pst = true;
                }
            },
            Ok(Event::End(ref e)) => {
                if e.name() == b"url" {
                    url = false;
                } else if e.name() == b"playlist_create_url" {
                    pst = false;
                }
            },
            Ok(Event::Text(ref e)) => {
                if url && pst {
                    let v = e.unescape_and_decode(&reader).unwrap();
                    urls.push(v);
                }
            },
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }
    urls
}

fn check_attr(reader: &Reader<&[u8]>, attrs: quick_xml::events::attributes::Attributes, attr_name: &str, attr_value: &str) -> bool {
    let sn = String::from(attr_name);
    let sv = String::from(attr_value);
    for attr in attrs {
        let a = attr.unwrap();
        let k = String::from_utf8(a.key.to_vec()).unwrap();
        if k == sn {
            let val = a.unescape_and_decode_value(reader).unwrap();
            if val == sv {
                return true;
            }
        }
    }
    false
}

fn get_auth_token(area_id: &String) -> eyre::Result<String> {
    let re = Regex::new(r"JP[1-47]").unwrap();
    if re.is_match(&area_id) {
        let client = reqwest::blocking::Client::new();
        let response1 = client.get("https://radiko.jp/v2/api/auth1")
            .header("X-Radiko-App", "aSmartPhone7o")
            .header("X-Radiko-App-Version", "0.0.1")
            .header("X-Radiko-Device", "Rust.radiko")
            .header("X-Radiko-User", "dummy_user")
            .send()?;
        if response1.status().is_success() {
            let auth_token = response1.headers().get("X-Radiko-AuthToken").unwrap().to_str()?;
            let key_offset: usize = response1.headers().get("X-Radiko-KeyOffset").unwrap().to_str()?.parse()?;
            let key_length: usize = response1.headers().get("X-Radiko-KeyLength").unwrap().to_str()?.parse()?;
            let partial_key = get_partialkey(key_offset, key_length);
            let coordvalue = get_coordinates(area_id).unwrap();
            let coodinate = format!("{},{},gps", coordvalue.0, coordvalue.1);
            let response2 = client.get("https://radiko.jp/v2/api/auth2")
                .header("X-Radiko-App", "aSmartPhone7o")
                .header("X-Radiko-App-Version", "0.0.1")
                .header("X-Radiko-AuthToken", auth_token)
                .header("X-Radiko-Connection", "wifi")
                .header("X-Radiko-Device", "Rust.radiko")
                .header("X-Radiko-Location", coodinate)
                .header("X-Radiko-PartialKey", partial_key)
                .header("X-Radiko-User", "dummy_user")
                .send()?;
            if response2.status().is_success() {
                return Ok(String::from(auth_token));
            }
        }
    }
    Err(eyre::eyre!("Cannot get an auth token !"))
}

fn get_coordinates(area_id: &String) -> Option<(f32, f32)> {
    for i in 0..COORDI_NAMES.len() {
        if COORDI_NAMES[i].0 == area_id {
            return Some((COORDI_NAMES[i].1, COORDI_NAMES[i].2));
        }
    }
    None
}

fn get_partialkey(offset: usize, length: usize) -> String {
    let decoded = FULL_KEY.into_bytes();
    let pkey: Vec<u8> = decoded[offset..offset+length].to_vec();
    base64::encode(pkey)
}

fn get_area_id_4_station_id(station_id: &String) -> Option<String> {
    if let Ok(stations) = get_stations() {
        for map in stations {
            if let Some(id) = map.get("id") {
                if id == station_id {
                    if let Some(area_id) = map.get("area_id") {
                        return Some(String::from(area_id));
                    }
                }
            }
        }
    }
    None
}

fn get_auth_token_4_station_id(station_id: &String) -> Option<String> {
    if is_available_station_id(station_id) {
        if let Some(area_id) = get_area_id_4_station_id(station_id) {
            if let Ok(auth_token) = get_auth_token(&area_id) {
                return Some(auth_token);
            }
        }
    }
    None
}



