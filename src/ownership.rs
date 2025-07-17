use std::collections::HashSet;

use crate::Error;
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use k8s_openapi::Resource;
use kube::api::{ApiResource, DynamicObject, GroupVersionKind};
use kube::discovery::ApiCapabilities;
use kube::{Api, ResourceExt};
use log::{trace, warn};

macro_rules! gvk {
    ($struct:ident) => {
        GroupVersionKind::gvk($struct::GROUP, $struct::VERSION, $struct::KIND)
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceIdentifier {
    pub gvk: GroupVersionKind,
    pub name: String,
    pub namespace: Option<String>,
}

impl TryFrom<DynamicObject> for ResourceIdentifier {
    type Error = Error;

    fn try_from(object: DynamicObject) -> Result<Self, Self::Error> {
        if let Some(types) = object.types.as_ref() {
            Ok(ResourceIdentifier {
                gvk: GroupVersionKind::try_from(types)?,
                name: object.name_any(),
                namespace: object.namespace(),
            })
        } else {
            Err(format!(
                "DynamicObject {:?} does not have a GroupVersionKind",
                object.name_any()
            )
            .into())
        }
    }
}

impl ResourceIdentifier {
    pub fn from_owner_reference(
        owner_reference: &OwnerReference,
        namespace: Option<String>,
    ) -> Self {
        ResourceIdentifier {
            gvk: GroupVersionKind::from(owner_reference.clone()),
            name: owner_reference.name.clone(),
            namespace,
        }
    }

    pub fn from_pod(pod: &Pod) -> Self {
        ResourceIdentifier {
            gvk: gvk!(Pod),
            name: pod.name_any(),
            namespace: pod.namespace(),
        }
    }
}

type OwnerCache = std::collections::HashMap<ResourceIdentifier, HashSet<ResourceIdentifier>>;

pub struct OwnershipResolver {
    client: kube::Client,
    discovery: Option<kube::discovery::Discovery>,
    should_discover: bool,
    owners: OwnerCache,
}

impl OwnershipResolver {
    pub fn new(client: kube::Client) -> Self {
        OwnershipResolver {
            client,
            discovery: None,
            should_discover: true,
            owners: OwnerCache::new(),
        }
    }

    // Clear the resolutions cache and allow re-discovery of Kubernetes APIs
    // if needed.
    pub fn reset(&mut self) {
        trace!("Resetting ownership resolver");
        self.should_discover = true;
        self.owners = OwnerCache::new();
    }

    // Re-discover Kubernetes APIs if allowed.
    async fn discover(&mut self) -> Result<(), Error> {
        if self.should_discover {
            trace!("Discovering Kubernetes APIs");
            let discovery = kube::discovery::Discovery::new(self.client.clone())
                .run()
                .await?;
            self.discovery = Some(discovery);
            self.should_discover = false;
        } else {
            warn!("Redundant Kubernetes API discovery requested, ignoring...");
        }

        Ok(())
    }

    // Use the discovery cache to resolve a GroupVersionKind.
    async fn resolve_gvk(
        &mut self,
        gvk: &GroupVersionKind,
    ) -> Result<(ApiResource, ApiCapabilities), Error> {
        if let Some(api) = self.discovery.as_mut().and_then(|d| d.resolve_gvk(gvk)) {
            return Ok(api);
        }

        // If an API is not found for a given GroupVersionKind, this is
        // either the first run, or the API was recently added to the cluster,
        // or it is not available in the cluster.
        // Run the discovery process to update the cache of known APIs.
        self.discover().await?;

        if let Some(api) = self.discovery.as_mut().and_then(|d| d.resolve_gvk(gvk)) {
            Ok(api)
        } else {
            Err(format!("Could not resolve GroupVersionKind {:?}", gvk).into())
        }
    }

    // Use the discovery cache to resolve an API for a given GroupVersionKind
    // and namespace.
    async fn resolve_api(
        &mut self,
        gvk: &GroupVersionKind,
        namespace: &Option<String>,
    ) -> Result<Api<DynamicObject>, Error> {
        let (api_resource, api_capabilities) = self.resolve_gvk(gvk).await?;

        match api_capabilities.scope {
            kube::discovery::Scope::Namespaced => {
                if let Some(namespace) = namespace {
                    // If the API is namespaced, return the API for the given namespace.
                    Ok(Api::namespaced_with(
                        self.client.clone(),
                        namespace,
                        &api_resource,
                    ))
                } else {
                    // If the API is namespaced but no namespace is provided, return an error.
                    Err(format!(
                        "Cannot resolve namespaced API {:?} without a namespace",
                        gvk
                    )
                    .into())
                }
            }
            kube::discovery::Scope::Cluster => {
                // If the API is cluster-scoped, return the API without a namespace,
                // ignoring the namespace provided.
                Ok(Api::all_with(self.client.clone(), &api_resource))
            }
        }
    }

    // Use the discovery cache to resolve a DynamicObject from a
    // ResourceIdentifier.
    async fn resolve_object(
        &mut self,
        resource: &ResourceIdentifier,
    ) -> Result<DynamicObject, Error> {
        let api = self.resolve_api(&resource.gvk, &resource.namespace).await?;

        Ok(api.get(&resource.name).await?)
    }

    // Use the discovery cache to resolve a DynamicObject's owner
    // references to dynamic objects.
    async fn resolve_owner_references(
        &mut self,
        object: &DynamicObject,
    ) -> Result<Vec<DynamicObject>, Error> {
        let mut result = Vec::new();

        for owner_reference in object.owner_references() {
            let resource =
                ResourceIdentifier::from_owner_reference(owner_reference, object.namespace());
            result.push(self.resolve_object(&resource).await?);
        }

        Ok(result)
    }

    // Use the discovery cache and the ownership cache to resolve
    // a DynamicObject to its top-level owners, that is, traversing the
    // ownership hierarchy, the owners which are not owned by any other
    // object.
    pub async fn resolve_top_level_owners(
        &mut self,
        resource: &ResourceIdentifier,
    ) -> Result<HashSet<ResourceIdentifier>, Error> {
        #[derive(Clone, Debug)]
        enum QueueEntry {
            DynamicObject(DynamicObject),
            ResourceIdentifier(ResourceIdentifier),
        }

        impl From<DynamicObject> for QueueEntry {
            fn from(object: DynamicObject) -> Self {
                QueueEntry::DynamicObject(object)
            }
        }

        impl From<ResourceIdentifier> for QueueEntry {
            fn from(resource: ResourceIdentifier) -> Self {
                QueueEntry::ResourceIdentifier(resource)
            }
        }

        impl TryFrom<QueueEntry> for ResourceIdentifier {
            type Error = Error;

            fn try_from(entry: QueueEntry) -> Result<Self, Self::Error> {
                match entry {
                    QueueEntry::DynamicObject(object) => object.try_into(),
                    QueueEntry::ResourceIdentifier(resource) => Ok(resource),
                }
            }
        }

        impl QueueEntry {
            async fn resolve(
                self,
                resolver: &mut OwnershipResolver,
            ) -> Result<DynamicObject, Error> {
                match self {
                    QueueEntry::DynamicObject(object) => Ok(object),
                    QueueEntry::ResourceIdentifier(resource) => {
                        resolver.resolve_object(&resource).await
                    }
                }
            }
        }

        let mut result: HashSet<ResourceIdentifier> = HashSet::new();

        let mut queue: Vec<QueueEntry> = vec![resource.clone().into()];
        let mut seen: HashSet<ResourceIdentifier> = HashSet::new();

        while let Some(current_entry) = queue.pop() {
            let current_key = current_entry.clone().try_into()?;

            if seen.contains(&current_key) {
                // If we have already seen this entry, skip it to prevent cycles.
                // It's unclear whether Kubernetes resources can own each other
                // cyclically, but this prevents infinite loops in case they do.
                warn!(
                    "Skipping already seen ownership queue entry: {:?}",
                    current_key
                );
                continue;
            }

            seen.insert(current_key.clone());

            let is_cached = self.owners.contains_key(&current_key);

            // Fetch the owners for the current entry, either as resource identifiers
            // from the owners cache, or as dynamic objects from the Kubernetes API.
            let owner_entries: Vec<QueueEntry> = if is_cached {
                trace!("Using cached owners for {:?}", current_key);
                self.owners
                    .get(&current_key)
                    .unwrap()
                    .iter()
                    .cloned()
                    .map(QueueEntry::from)
                    .collect()
            } else {
                trace!("Resolving owners for {:?}", current_key);
                let current_object = current_entry.resolve(self).await?;
                let owner_objects = self.resolve_owner_references(&current_object).await?;
                owner_objects.into_iter().map(QueueEntry::from).collect()
            };

            // If there are no owner entries for the current entry, the current entry
            // is a top-level owner, so we add it to the result set.
            if owner_entries.is_empty() {
                trace!("Found root owner: {:?}", current_key);
                result.insert(current_key.clone());
            }

            // Add the owner entries, if any, to the queue.
            for owner_entry in owner_entries.iter().cloned() {
                trace!("Adding owner entry to queue: {:?}", owner_entry);
                queue.push(owner_entry);
            }

            // If the current entry is not cached, insert the owner references
            // into the cache.
            if !is_cached {
                let owner_references = owner_entries
                    .into_iter()
                    .map(|entry| entry.try_into())
                    .collect::<Result<HashSet<ResourceIdentifier>, Error>>()?;

                trace!(
                    "Caching owners for {:?}: {:?}",
                    current_key,
                    owner_references
                );
                self.owners.insert(current_key, owner_references);
            }
        }

        Ok(result)
    }
}
