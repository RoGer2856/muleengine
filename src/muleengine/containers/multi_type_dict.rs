use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
    sync::Arc,
};

pub struct MultiTypeDict {
    storage: BTreeMap<TypeId, Arc<dyn Any>>,
}

impl MultiTypeDict {
    pub fn new() -> Self {
        Self {
            storage: BTreeMap::new(),
        }
    }

    pub fn insert<ItemType>(&mut self, item: ItemType) -> Option<Arc<ItemType>>
    where
        ItemType: Any,
    {
        let type_id = TypeId::of::<ItemType>();
        let item = Arc::new(item);

        Self::downcast_optional_item(self.storage.insert(type_id, item))
    }

    pub fn insert_any<ItemType>(
        &mut self,
        item: impl Any,
        type_id: TypeId,
    ) -> Option<Arc<dyn Any>> {
        self.storage.insert(type_id, Arc::new(item))
    }

    pub fn get_item_ref<ItemType>(&self) -> Option<Arc<ItemType>>
    where
        ItemType: Any,
    {
        let type_id = TypeId::of::<ItemType>();

        Self::downcast_optional_item(self.get_item_ref_any(type_id))
    }

    pub fn get_item_ref_any(&self, type_id: TypeId) -> Option<Arc<dyn Any>> {
        self.storage.get(&type_id).map(|item| item.clone())
    }

    pub fn remove<ItemType>(&mut self) -> Option<Arc<ItemType>>
    where
        ItemType: Any,
    {
        let type_id = TypeId::of::<ItemType>();

        Self::downcast_optional_item(self.remove_by_type_id(type_id))
    }

    pub fn remove_by_type_id(&mut self, type_id: TypeId) -> Option<Arc<dyn Any>> {
        self.storage.remove(&type_id)
    }

    fn downcast_optional_item<ItemType>(item: Option<Arc<dyn Any>>) -> Option<Arc<ItemType>>
    where
        ItemType: Any,
    {
        item.map(|item| {
            if (*item).is::<ItemType>() {
                let ptr = Arc::into_raw(item).cast::<ItemType>();

                Some(unsafe { Arc::from_raw(ptr) })
            } else {
                None
            }
        })
        .flatten()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::MultiTypeDict;

    #[derive(Debug, Eq, PartialEq)]
    struct A {
        value: String,
    }

    #[derive(Debug, Eq, PartialEq)]
    struct B {
        value: String,
    }

    #[test]
    fn store_and_remove() {
        let mut dict = MultiTypeDict::new();

        assert!(dict
            .insert(A {
                value: "A0".to_string(),
            })
            .is_none());

        assert_eq!(
            dict.insert(A {
                value: "A1".to_string(),
            }),
            Some(Arc::new(A {
                value: "A0".to_string(),
            }))
        );

        assert!(dict
            .insert(B {
                value: "B".to_string(),
            })
            .is_none());

        assert_eq!(
            dict.get_item_ref::<A>(),
            Some(Arc::new(A {
                value: "A1".to_string(),
            }))
        );

        assert_eq!(
            dict.get_item_ref::<B>(),
            Some(Arc::new(B {
                value: "B".to_string(),
            }))
        );

        assert_eq!(
            dict.remove::<A>(),
            Some(Arc::new(A {
                value: "A1".to_string(),
            }))
        );

        assert!(dict.get_item_ref::<A>().is_none());

        assert_eq!(
            dict.remove::<B>(),
            Some(Arc::new(B {
                value: "B".to_string(),
            }))
        );

        assert!(dict.get_item_ref::<B>().is_none());
    }
}
