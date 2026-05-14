//! Genesis Marketplace — asset store, plugin store, game store, AI recommendations
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ── Common ────────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Price { pub cents:u64, pub currency:String, pub sale:Option<Sale> }
impl Price {
    pub fn free()->Self{Self{cents:0,currency:"USD".to_string(),sale:None}}
    pub fn usd(d:f32)->Self{Self{cents:(d*100.0) as u64,currency:"USD".to_string(),sale:None}}
    pub fn is_free(&self)->bool{self.cents==0}
    pub fn display(&self)->String{
        if self.is_free(){return "Free".to_string();}
        if let Some(s)=&self.sale{
            let d=(self.cents as f32*(1.0-s.pct/100.0)) as u64;
            return format!("${:.2} ({}% off)",d as f32/100.0,s.pct as u32);
        }
        format!("${:.2}",self.cents as f32/100.0)
    }
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Sale{pub pct:f32,pub starts:DateTime<Utc>,pub ends:DateTime<Utc>,pub label:Option<String>}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Publisher {
    pub id:String, pub name:String, pub display:String,
    pub verified:bool, pub trusted:bool, pub since:DateTime<Utc>,
    pub sales:u64, pub rating:f32, pub revenue_share:f32,
    pub website:Option<String>, pub social:HashMap<String,String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Review {
    pub id:String, pub reviewer:String, pub stars:u8,
    pub title:String, pub body:String, pub helpful:u32,
    pub verified_purchase:bool, pub created:DateTime<Utc>,
}

// ── Asset Package ─────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AssetPackage {
    pub id:String, pub title:String, pub description:String,
    pub publisher:Publisher, pub category:AssetCategory,
    pub tags:Vec<String>, pub price:Price, pub license:AssetLicense,
    pub version:String, pub min_engine:String, pub rating:f32,
    pub reviews:Vec<Review>, pub downloads:u64, pub size_bytes:u64,
    pub featured:bool, pub ai_quality_score:f32,
    pub commercial_use:bool, pub includes_source:bool,
    pub created:DateTime<Utc>, pub updated:DateTime<Utc>,
    pub screenshots:Vec<String>, pub changelog:Vec<ChangeEntry>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum AssetCategory {
    Mesh3D{tris:u32,rigged:bool,animated:bool},
    Texture{res:[u32;2],pbr:bool,seamless:bool},
    Material{kind:String}, VfxEffect{realtime:bool},
    Audio{secs:f32,loops:bool}, Music{genre:String,bpm:f32},
    Animation{clips:u32,retargetable:bool}, Shader{pipelines:Vec<String>},
    Font{styles:u32}, ScriptLibrary{count:u32,lang:String},
    Template{genre:String,complete:bool}, AiVoice{lang:String,style:String},
    AiModelPack{count:u32,task:String}, Blueprint{nodes:u32},
    ScenePreset{objects:u32}, UiKit{components:u32},
    CharacterPack{count:u32}, EnvironmentPack{biome:String,objects:u32},
    PropPack{theme:String,count:u32}, Bundle{includes:Vec<String>},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum AssetLicense { Standard, Attribution, Personal, ExtendedCommercial, PublicDomain, Custom{url:String} }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChangeEntry{pub version:String,pub date:DateTime<Utc>,pub changes:Vec<String>}

// ── Plugin Listing ────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PluginListing {
    pub id:String, pub name:String, pub slug:String,
    pub description:String, pub publisher:Publisher,
    pub kind:PluginKind, pub category:String,
    pub price:Price, pub version:String, pub min_engine:String,
    pub rating:f32, pub installs:u64, pub featured:bool,
    pub open_source:bool, pub source_url:Option<String>,
    pub binary_url:String, pub size_bytes:u64,
    pub permissions:Vec<String>, pub screenshots:Vec<String>,
    pub reviews:Vec<Review>, pub created:DateTime<Utc>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum PluginKind {
    EngineExtension{panels:bool,nodes:bool},
    AiAgentPack{count:u32,spec:String},
    PlatformTarget{platform:String},
    MonetizationTool, Analytics{provider:String},
    Importer{format:String}, Exporter{format:String},
    IntegrationPack{service:String}, DevTool{kind:String},
}

// ── Game Listing (PS5-style store) ────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct GameListing {
    pub id:String, pub title:String, pub slug:String,
    pub tagline:String, pub description:String,
    pub developer:Publisher, pub genre:Vec<String>, pub tags:Vec<String>,
    pub price:Price, pub monetization:MonetizationModel,
    pub cover_art:String, pub hero_art:String, pub icon:String,
    pub screenshots:Vec<String>, pub trailer_url:Option<String>,
    pub platforms:Vec<String>, pub languages:Vec<String>,
    pub status:ReleaseStatus, pub release_date:Option<DateTime<Utc>>,
    pub early_access:bool, pub content_warnings:Vec<String>,
    pub rating_board:Vec<(String,String)>, // board, rating
    pub rating:f32, pub reviews:Vec<Review>,
    pub owners:u64, pub wishlists:u64, pub peak_players:u64,
    pub achievements:Vec<Achievement>, pub cloud_save:bool,
    pub controller_support:ControllerSupport,
    pub multiplayer:MultiplayerInfo, pub dlc:Vec<DlcListing>,
    pub in_app_purchases:bool, pub revenue_total_cents:u64,
    pub units_sold:u64, pub created:DateTime<Utc>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MonetizationModel {
    Premium{price:Price}, Free, Freemium{iap:bool,ads:bool},
    Subscription{monthly:Price,yearly:Price},
    PayWhatYouWant{min:Price,suggested:Price},
    EarlyAccess{ea_price:Price,final_price:Price},
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ReleaseStatus { Draft, InReview, ComingSoon, EarlyAccess, Released, Delisted }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Achievement{pub id:String,pub name:String,pub desc:String,pub hidden:bool,pub points:u32,pub unlock_pct:f32}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ControllerSupport{None,Partial,Full}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MultiplayerInfo{pub pvp:bool,pub coop:bool,pub max_players:u32,pub dedicated_servers:bool,pub cross_play:bool}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DlcListing{pub id:String,pub title:String,pub price:Price,pub released:bool,pub kind:String,pub size_gb:f32}

// ── User Library & Cart ───────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct UserLibrary {
    pub assets:Vec<OwnedItem>, pub plugins:Vec<OwnedItem>, pub games:Vec<OwnedItem>,
    pub spent_cents:u64, pub points:u64,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct OwnedItem {
    pub item_id:String, pub kind:String, pub purchased:DateTime<Utc>,
    pub paid_cents:u64, pub installed:bool, pub version:String,
    pub update_available:bool,
}

#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct Cart { pub items:Vec<CartItem>, pub coupon:Option<String>, pub discount_pct:f32 }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CartItem{pub id:String,pub kind:String,pub cents:u64}
impl Cart {
    pub fn subtotal(&self)->u64{self.items.iter().map(|i|i.cents).sum()}
    pub fn total(&self)->u64{
        let s=self.subtotal();
        if self.discount_pct>0.0{(s as f32*(1.0-self.discount_pct/100.0)) as u64}else{s}
    }
    pub fn add(&mut self,id:&str,kind:&str,cents:u64){
        if !self.items.iter().any(|i|i.id==id){
            self.items.push(CartItem{id:id.to_string(),kind:kind.to_string(),cents});
        }
    }
    pub fn remove(&mut self,id:&str){self.items.retain(|i|i.id!=id);}
}

// ── AI Recommendations ────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Recommendation{pub id:String,pub reason:String,pub score:f32,pub priority:String}

// ── Marketplace Manager ───────────────────────────────────────────
pub struct MarketplaceManager {
    pub assets:HashMap<String,AssetPackage>,
    pub plugins:HashMap<String,PluginListing>,
    pub games:HashMap<String,GameListing>,
    pub library:UserLibrary, pub cart:Cart,
    pub wishlist:Vec<String>,
    pub revenue_share_pct:f32, // 12% to Genesis
    pub base_url:String,
}

impl MarketplaceManager {
    pub fn new()->Self{
        Self{ assets:HashMap::new(), plugins:HashMap::new(), games:HashMap::new(),
              library:UserLibrary::default(), cart:Cart::default(),
              wishlist:Vec::new(), revenue_share_pct:12.0,
              base_url:"https://marketplace.genesis-engine.io".to_string() }
    }
    pub fn search_assets(&self,q:&str,max_price:Option<u64>)->Vec<&AssetPackage>{
        let ql=q.to_lowercase();
        self.assets.values().filter(|a|{
            let matches_q=a.title.to_lowercase().contains(&ql)||a.tags.iter().any(|t|t.to_lowercase().contains(&ql));
            let matches_p=max_price.map(|p|a.price.cents<=p).unwrap_or(true);
            matches_q&&matches_p
        }).collect()
    }
    pub fn search_games(&self,q:&str)->Vec<&GameListing>{
        let ql=q.to_lowercase();
        self.games.values().filter(|g| g.title.to_lowercase().contains(&ql)||g.tags.iter().any(|t|t.to_lowercase().contains(&ql))).collect()
    }
    pub fn is_owned(&self,id:&str)->bool{
        self.library.assets.iter().any(|i|i.item_id==id)||
        self.library.plugins.iter().any(|i|i.item_id==id)||
        self.library.games.iter().any(|i|i.item_id==id)
    }
    pub fn add_to_wishlist(&mut self,id:&str){
        if !self.wishlist.contains(&id.to_string()){self.wishlist.push(id.to_string());}
    }
    pub fn developer_revenue_pct(&self)->f32{100.0-self.revenue_share_pct}
    pub fn asset_count(&self)->usize{self.assets.len()}
    pub fn plugin_count(&self)->usize{self.plugins.len()}
    pub fn game_count(&self)->usize{self.games.len()}
}
