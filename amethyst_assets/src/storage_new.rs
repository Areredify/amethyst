use atelier_loader::LoadHandle;
use crossbeam_queue::SegQueue;
use std::collections::HashMap;

use crate::handle_new::AssetHandle;

struct AssetState<A> {
    version: u32,
    asset: A,
}

/// An asset storage, storing the actual assets and allocating handles to them.
///
/// # Type Parameters
///
/// * `A`: Asset Rust type.
pub struct AssetStorage<A> {
    assets: HashMap<LoadHandle, AssetState<A>>,
    uncommitted: HashMap<LoadHandle, AssetState<A>>,
    to_drop: SegQueue<A>,
}

impl<A> AssetStorage<A> {
    /// Creates a new asset storage.
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn update_asset(&mut self, handle: LoadHandle, asset: A, version: u32) {
        if let Some(data) = self.uncommitted.remove(&handle) {
            // uncommitted data already exists for the handle, drop it
            self.to_drop.push(data.asset);
        }
        self.uncommitted
            .insert(handle, AssetState { version, asset });
    }

    pub(crate) fn remove_asset(&mut self, handle: LoadHandle) {
        if let Some(data) = self.uncommitted.remove(&handle) {
            self.to_drop.push(data.asset);
        }
        if let Some(data) = self.assets.remove(&handle) {
            self.to_drop.push(data.asset);
        }
    }

    pub(crate) fn commit_asset(&mut self, handle: LoadHandle, version: u32) {
        if let Some(data) = self.uncommitted.remove(&handle) {
            if data.version != version {
                panic!("attempted to commit asset version which mismatches with existing uncommitted version")
            }
            if let Some(existing) = self.assets.remove(&handle) {
                // data already exists for the handle, drop it
                self.to_drop.push(existing.asset);
            }
            self.assets.insert(
                handle,
                AssetState {
                    version,
                    asset: data.asset,
                },
            );
        } else {
            panic!("attempted to commit asset which doesn't exist");
        }
    }

    /// Returns the asset for the given handle, or `None` if has not completed loading.
    ///
    /// # Parameters
    ///
    /// * `handle`: Handle of the asset.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset handle type.
    pub fn get<T: AssetHandle>(&self, handle: &T) -> Option<&A> {
        self.assets.get(&handle.load_handle()).map(|a| &a.asset)
    }

    /// Get an asset mutably from a given asset handle.
    pub fn get_mut<T: AssetHandle>(&mut self, handle: &T) -> Option<&mut A> {
        self.assets
            .get_mut(&handle.load_handle())
            .map(|a| &mut a.asset)
    }

    /// Returns the version of a loaded asset, or `None` if has not completed loading.
    ///
    /// # Parameters
    ///
    /// * `handle`: Handle of the asset.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset handle type.
    pub fn get_version<T: AssetHandle>(&self, handle: &T) -> Option<u32> {
        self.assets.get(&handle.load_handle()).map(|a| a.version)
    }

    /// Returns the loaded asset and its version, or `None` if has not completed loading.
    ///
    /// # Parameters
    ///
    /// * `handle`: Handle of the asset.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset handle type.
    pub fn get_asset_with_version<T: AssetHandle>(&self, handle: &T) -> Option<(&A, u32)> {
        self.assets
            .get(&handle.load_handle())
            .map(|a| (&a.asset, a.version))
    }

    /// Process finished asset data and maintain the storage.
    ///
    /// This calls the `drop_fn` function for assets that were removed from the storage.
    ///
    /// # Parameters
    ///
    /// * `drop_fn`: The function to invoke with the asset.
    ///
    /// # Type Parameters
    ///
    /// * `D`: Drop function type.
    pub fn process_custom_drop<D>(&mut self, mut drop_fn: D)
    where
        D: FnMut(A),
    {
        while let Ok(asset) = self.to_drop.pop() {
            drop_fn(asset);
        }
    }
}

impl<A> Default for AssetStorage<A> {
    fn default() -> Self {
        AssetStorage {
            assets: Default::default(),
            uncommitted: Default::default(),
            to_drop: SegQueue::new(),
        }
    }
}