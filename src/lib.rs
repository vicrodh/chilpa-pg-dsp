//! I supply data-only processing presets through the standard WASM ABI.
//! The host validates the catalog and evaluates the filters outside WASM.
use chilpa_plugin_sdk::plugin;
use serde::Deserialize;
use serde_json::Value;
mod catalog;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request { protocol: u32 }

pub fn presets(request: Request) -> Result<Value, String> {
    if !matches!(request.protocol, 1 | 2) { return Err("Unsupported DSP protocol.".into()); }
    let catalog: Value = serde_json::from_str(include_str!("../presets.json")).map_err(|_| "The DSP preset catalog is invalid.")?;
    if request.protocol == 2 { Ok(serde_json::json!({"protocol":2,"catalog":catalog,"search":true})) }
    else { Ok(catalog) }
}

plugin! { api_version: 1, chilpa_dsp_presets => presets, chilpa_dsp_search => catalog::search }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_mismatch_and_unknown_request_fields_are_rejected() {
        assert!(presets(Request { protocol: 3 }).is_err());
        assert!(serde_json::from_str::<Request>("{}").is_err());
        assert!(serde_json::from_str::<Request>(r#"{"protocol":1,"path":"/music"}"#).is_err());
    }
    #[test]
    fn catalog_has_unique_usable_presets_and_no_runtime_permissions() {
        let reply = presets(Request { protocol: 1 }).unwrap();
        let presets = reply["presets"].as_array().unwrap();
        assert!(presets.len() >= 40);
        let mut ids = std::collections::BTreeSet::new();
        for p in presets {
            assert!(ids.insert(p["id"].as_str().unwrap()));
            assert!(!p["name"].as_str().unwrap().is_empty());
            assert!(!p["description"].as_str().unwrap().is_empty());
            assert!(p["filters"].as_array().unwrap().len() <= 10);
        }
        let manifest: Value = serde_json::from_str(include_str!("../manifest.json")).unwrap();
        assert_eq!(manifest["kind"], "dsp");
        assert_eq!(manifest["capabilities"], serde_json::json!([{"type":"process_audio"}]));
    }

    #[test]
    fn every_preset_has_spanish_labels_and_english_is_the_default() {
        let reply = presets(Request { protocol: 1 }).unwrap();
        let spanish = reply["localizations"]["es"].as_object().unwrap();
        assert_eq!(spanish.len(), reply["presets"].as_array().unwrap().len());
        for preset in reply["presets"].as_array().unwrap() {
            let labels = &spanish[preset["id"].as_str().unwrap()];
            for key in ["name", "category", "description"] { assert!(!labels[key].as_str().unwrap().is_empty()); }
        }
        let soundbar = reply["presets"].as_array().unwrap().iter().find(|p| p["id"] == "soundbar-21").unwrap();
        assert_eq!(soundbar["name"], "Soundbar 2.1 · clear voices and controlled bass");
        assert!(soundbar["description"].as_str().unwrap().starts_with("Generic adjustment"));
        assert!(spanish["soundbar-21"]["description"].as_str().unwrap().starts_with("Ajuste genérico"));
    }
}
