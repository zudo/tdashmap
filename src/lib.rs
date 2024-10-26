use dashmap::DashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
#[derive(Clone, Debug, Default)]
pub struct TDashMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    map: DashMap<K, (V, Instant)>,
    ttl: Duration,
}
impl<K, V> TDashMap<K, V>
where
    K: Eq + std::hash::Hash + Debug,
    V: Debug,
{
    pub fn new(ttl: Duration) -> TDashMap<K, V> {
        TDashMap {
            map: DashMap::new(),
            ttl,
        }
    }
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.map.get(key).and_then(|entry| {
            if entry.value().1.elapsed() < self.ttl {
                Some(entry.value().0.clone())
            } else {
                None
            }
        })
    }
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.map
            .insert(key, (value, Instant::now()))
            .and_then(|(value, timestamp)| {
                if timestamp.elapsed() < self.ttl {
                    Some(value)
                } else {
                    None
                }
            })
    }
    pub fn remove(&self, key: &K) -> Option<V> {
        self.map.remove(key).and_then(|(_, (value, timestamp))| {
            if timestamp.elapsed() < self.ttl {
                Some(value)
            } else {
                None
            }
        })
    }
    pub fn cleanup(&self) {
        self.map
            .retain(|_key, (_value, timestamp)| timestamp.elapsed() < self.ttl);
    }
}
impl<K, V> TDashMap<K, V>
where
    K: Eq + std::hash::Hash + Debug + Send + Sync + 'static,
    V: Debug + Send + Sync + 'static,
{
    pub fn spawn_cleanup(self: Arc<Self>, period: Duration) {
        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(period);
            cleanup_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                cleanup_interval.tick().await;
                self.cleanup();
            }
        });
    }
}
