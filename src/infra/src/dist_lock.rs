// Copyright 2023 Zinc Labs Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use config::CONFIG;

use crate::{
    db::{etcd, nats},
    errors::Result,
};

pub struct Locker(LockerStore);

enum LockerStore {
    Etcd(etcd::Locker),
    Nats(nats::Locker),
}

/// lock key in etcd, wait_ttl is 0 means wait forever
#[inline(always)]
pub async fn lock(key: &str, wait_ttl: u64) -> Result<Option<Locker>> {
    if CONFIG.common.local_mode || !CONFIG.common.feature_query_queue_enabled {
        return Ok(None);
    }
    match CONFIG.common.cluster_coordinator.as_str() {
        "nats" => {
            let mut lock = nats::Locker::new(key);
            lock.lock(wait_ttl).await?;
            Ok(Some(Locker(LockerStore::Nats(lock))))
        }
        _ => {
            let mut lock = etcd::Locker::new(key);
            lock.lock(wait_ttl).await?;
            Ok(Some(Locker(LockerStore::Etcd(lock))))
        }
    }
}

#[inline(always)]
pub async fn unlock(locker: &Option<Locker>) -> Result<()> {
    if let Some(locker) = locker {
        match &locker.0 {
            LockerStore::Etcd(locker) => locker.unlock().await,
            LockerStore::Nats(locker) => locker.unlock().await,
        }
    } else {
        Ok(())
    }
}
