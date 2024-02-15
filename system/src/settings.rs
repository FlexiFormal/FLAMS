use immt_api::Str;
use immt_api::utils::HMap;

#[derive(Default)]
pub struct Settings {
    scopes:parking_lot::RwLock<HMap<Str,SettingsScope>>
}
impl Settings {
    /*pub fn get_all(&self) -> impl Iterator<Item=((&str,&str),&SettingsValue)> {
        self.scopes.read().iter().flat_map(|(k,v)| v.settings.iter().map(|(k2,v2)| ((&**k,&**k2),v2)))
    }*/
    pub fn set<const NUM:usize>(&self,settings:[(Str,Str,SettingsValue);NUM]) {
        let mut lock = self.scopes.write();
        for (scope,key,value) in settings.into_iter() {
            let scope = lock.entry(scope).or_insert_with(|| SettingsScope{settings:HMap::default()});
            scope.settings.insert(key,value);
        }
    }
    pub fn set_default<S1:Into<Str>,S2:Into<Str>,const NUM:usize>(&self,settings:[(S1,S2,SettingsValue);NUM]) {
        let mut lock = self.scopes.write();
        for (scope,key,value) in settings.into_iter() {
            let scope = lock.entry(scope.into()).or_insert_with(|| SettingsScope{settings:HMap::default()});
            scope.settings.entry(key.into()).or_insert(value);
        }
    }
    pub fn get<R,const NUM:usize>(&self,keys:[(&str,&str);NUM],f:impl FnOnce([Option<&SettingsValue>;NUM]) -> R) -> R {
        let lock = self.scopes.read();
        let mut res = [None;NUM];
        for (i,(scope,key)) in keys.iter().enumerate() {
            if let Some(s) = lock.get(*scope) {
                res[i] = s.settings.get(*key);
            }
        }
        f(res)
        //lock.get(scope).and_then(|s| f(s.settings.get(key).as_ref().copied())).unwrap_or_else(f(None))
    }
}

pub struct SettingsScope {
    settings:HMap<Str,SettingsValue>
}

pub enum SettingsValue {
    String(Str),
    Integer(i64),
    PositiveInteger(u64),
    Float(f64),
    Boolean(bool),
}