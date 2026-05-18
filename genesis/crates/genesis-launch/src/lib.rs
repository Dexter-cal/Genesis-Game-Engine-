//! Genesis Launcher — project browser, recent projects, version manager, updates
use serde::{Serialize,Deserialize};
use std::path::PathBuf;
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RecentProject {
    pub id:String, pub name:String, pub path:PathBuf,
    pub last_opened:DateTime<Utc>, pub engine_version:String,
    pub thumbnail:Option<PathBuf>, pub description:String,
    pub template:String, pub pinned:bool, pub missing:bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct EngineVersion {
    pub version:String, pub channel:String, pub path:PathBuf,
    pub installed:DateTime<Utc>, pub size_mb:u64, pub default:bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AvailableUpdate {
    pub version:String, pub channel:String, pub size_mb:u64,
    pub changelog_url:String, pub critical:bool, pub date:DateTime<Utc>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NewsItem {
    pub id:String, pub title:String, pub summary:String,
    pub url:String, pub category:String, pub date:DateTime<Utc>,
    pub image:Option<String>, pub read:bool,
}

#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct LauncherState {
    pub recent:Vec<RecentProject>, pub versions:Vec<EngineVersion>,
    pub active_version:String, pub check_updates:bool,
    pub last_check:Option<DateTime<Utc>>, pub update:Option<AvailableUpdate>,
    pub news:Vec<NewsItem>, pub launches:u64, pub first:bool,
    pub onboarded:bool, pub username:Option<String>, pub theme:String,
}

pub struct LauncherManager {
    pub state:LauncherState,
    pub config_path:PathBuf,
    pub projects_dir:PathBuf,
}

impl LauncherManager {
    pub fn new(config_dir:&std::path::Path)->Self{
        let config_path=config_dir.join("launcher.json");
        let state = std::fs::read_to_string(&config_path)
            .ok().and_then(|s|serde_json::from_str(&s).ok())
            .unwrap_or_else(||LauncherState{
                active_version:env!("CARGO_PKG_VERSION").to_string(),
                check_updates:true, first:true, theme:"dark".to_string(),
                ..Default::default()
            });
        let projects_dir = dirs::home_dir().unwrap_or_default().join("genesis-projects");
        Self{state,config_path,projects_dir}
    }

    pub fn save(&self)->Result<(),Box<dyn std::error::Error>>{
        let json=serde_json::to_string_pretty(&self.state)?;
        std::fs::create_dir_all(self.config_path.parent().unwrap_or(std::path::Path::new(".")))?;
        std::fs::write(&self.config_path,json)?;
        Ok(())
    }

    pub fn add_recent(&mut self,p:RecentProject){
        self.state.recent.retain(|r|r.path!=p.path);
        self.state.recent.insert(0,p);
        self.state.recent.truncate(20);
    }

    pub fn remove_recent(&mut self,path:&PathBuf){
        self.state.recent.retain(|p|&p.path!=path);
    }

    pub fn pin(&mut self,path:&PathBuf,pin:bool){
        if let Some(p)=self.state.recent.iter_mut().find(|p|&p.path==path){p.pinned=pin;}
    }

    pub fn pinned(&self)->Vec<&RecentProject>{ self.state.recent.iter().filter(|p|p.pinned).collect() }
    pub fn unpinned(&self)->Vec<&RecentProject>{ self.state.recent.iter().filter(|p|!p.pinned).collect() }

    pub fn launch(&self,path:&PathBuf,scene:Option<&str>,headless:bool)->std::io::Result<std::process::Child>{
        let mut cmd=std::process::Command::new("genesis");
        cmd.arg("--project").arg(path);
        if let Some(s)=scene{cmd.arg("--scene").arg(s);}
        if headless{cmd.arg("--headless");}
        cmd.spawn()
    }

    pub fn tick_launch(&mut self){
        self.state.launches+=1;
        self.state.first=false;
        let _ = self.save();
    }

    pub fn has_update(&self)->bool{self.state.update.is_some()}
    pub fn recent_count(&self)->usize{self.state.recent.len()}
    pub fn version_count(&self)->usize{self.state.versions.len()}
}
