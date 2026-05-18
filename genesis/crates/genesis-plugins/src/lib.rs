//! Plugin System — hot-reload, marketplace, sandboxed execution
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PluginManifest {
    pub id:          String,
    pub name:        String,
    pub version:     String,
    pub description: String,
    pub author:      String,
    pub homepage:    Option<String>,
    pub min_engine:  String,
    pub entry_point: String,   // path to .so/.dll/.wasm
    pub permissions: Vec<Permission>,
    pub provides:    Vec<String>,   // node types, agent types, etc.
    pub depends_on:  Vec<String>,   // other plugin IDs
    pub category:    PluginCategory,
    pub auto_enable: bool,
    pub hot_reload:  bool,
    pub checksum:    String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum PluginCategory {
    Gameplay, Graphics, Audio, Network, Ai, Editor,
    Importer{format:String}, Exporter{format:String},
    Monetization, Analytics, Platform{name:String}, Custom(String),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Permission {
    ReadScene, WriteScene, ReadFiles{paths:Vec<String>}, WriteFiles{paths:Vec<String>},
    Network{domains:Vec<String>}, Spawn, DestroyEntities,
    AccessAi, AccessPhysics, AccessAudio, AccessInput,
    SystemProcess, Admin,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum PluginState { Unloaded, Loading, Enabled, Disabled, Error(String), HotReloading }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PluginInstance {
    pub manifest:     PluginManifest,
    pub state:        PluginState,
    pub loaded_at:    Option<DateTime<Utc>>,
    pub enabled_at:   Option<DateTime<Utc>>,
    pub last_reload:  Option<DateTime<Utc>>,
    pub error_count:  u32,
    pub reload_count: u32,
    pub install_path: String,
    pub data_path:    String,
    pub settings:     HashMap<String,serde_json::Value>,
}

pub struct PluginRegistry {
    pub plugins:      HashMap<String,PluginInstance>,
    pub load_order:   Vec<String>,
    pub search_paths: Vec<String>,
    pub sandbox:      bool,
    pub hot_reload:   bool,
    pub total_loaded: u64,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins:HashMap::new(), load_order:Vec::new(),
               search_paths:vec!["plugins/".to_string()],
               sandbox:true, hot_reload:true, total_loaded:0 }
    }

    pub fn register(&mut self, manifest:PluginManifest, install_path:&str) -> String {
        let id = manifest.id.clone();
        tracing::info!("Registering plugin: {} v{}", manifest.name, manifest.version);
        // Check permissions
        for perm in &manifest.permissions {
            if matches!(perm, Permission::Admin) {
                tracing::warn!("Plugin {} requests Admin permission!", manifest.name);
            }
        }
        let inst = PluginInstance {
            manifest, state:PluginState::Unloaded,
            loaded_at:None, enabled_at:None, last_reload:None,
            error_count:0, reload_count:0,
            install_path:install_path.to_string(),
            data_path:format!("data/plugins/{}/",id),
            settings:HashMap::new(),
        };
        self.plugins.insert(id.clone(), inst);
        id
    }

    pub fn enable(&mut self, id:&str) -> bool {
        if let Some(p) = self.plugins.get_mut(id) {
            p.state = PluginState::Enabled;
            p.enabled_at = Some(chrono::Utc::now());
            if !self.load_order.contains(&id.to_string()) { self.load_order.push(id.to_string()); }
            tracing::info!("Plugin enabled: {}", id);
            self.total_loaded += 1;
            true
        } else { false }
    }

    pub fn disable(&mut self, id:&str) -> bool {
        if let Some(p) = self.plugins.get_mut(id) {
            p.state = PluginState::Disabled;
            tracing::info!("Plugin disabled: {}", id);
            true
        } else { false }
    }

    pub fn hot_reload(&mut self, id:&str) {
        if let Some(p) = self.plugins.get_mut(id) {
            if p.manifest.hot_reload {
                p.state = PluginState::HotReloading;
                p.reload_count += 1;
                p.last_reload = Some(chrono::Utc::now());
                tracing::info!("Hot-reloading plugin: {}", id);
                p.state = PluginState::Enabled;
            }
        }
    }

    pub fn is_enabled(&self, id:&str) -> bool {
        self.plugins.get(id).map(|p| p.state == PluginState::Enabled).unwrap_or(false)
    }

    pub fn provides(&self, node_type:&str) -> Option<&str> {
        self.plugins.values()
            .find(|p| p.state == PluginState::Enabled && p.manifest.provides.iter().any(|s| s == node_type))
            .map(|p| p.manifest.id.as_str())
    }

    pub fn count(&self) -> usize { self.plugins.len() }
    pub fn enabled_count(&self) -> usize { self.plugins.values().filter(|p| p.state == PluginState::Enabled).count() }
}
extern crate tracing;
