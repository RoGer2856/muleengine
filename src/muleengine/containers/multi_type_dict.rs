use std::{
    any::{Any, TypeId},
    collections::{btree_map::Iter, BTreeMap},
    sync::Arc,
};

pub struct MultiTypeDictItem<ItemType: ?Sized> {
    type_id: TypeId,
    item: Arc<ItemType>,
}

impl<ItemType: ?Sized> Clone for MultiTypeDictItem<ItemType> {
    fn clone(&self) -> Self {
        Self {
            type_id: self.type_id,
            item: self.item.clone(),
        }
    }
}

pub struct MultiTypeDict {
    storage: BTreeMap<TypeId, MultiTypeDictItem<dyn Any>>,
}

pub struct MultiTypeDictIterator<'a> {
    inner_iterator: Iter<'a, TypeId, MultiTypeDictItem<dyn Any>>,
}

pub struct MultiTypeDictInsertResult<ItemType: ?Sized> {
    pub new_item: MultiTypeDictItem<ItemType>,
    pub old_item: Option<MultiTypeDictItem<ItemType>>,
}

impl<'a> Iterator for MultiTypeDictIterator<'a> {
    type Item = MultiTypeDictItem<dyn Any>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iterator.next().map(|value| value.1.clone())
    }
}

impl MultiTypeDict {
    pub fn new() -> Self {
        Self {
            storage: BTreeMap::new(),
        }
    }

    pub fn insert<ItemType>(&mut self, item: ItemType) -> MultiTypeDictInsertResult<ItemType>
    where
        ItemType: Any,
    {
        let type_id = TypeId::of::<ItemType>();

        let result = self.insert_any(item, type_id);
        if let Some(new_item) = result.new_item.downcast() {
            if let Some(old_item) = result.old_item {
                if let Some(old_item) = old_item.downcast() {
                    MultiTypeDictInsertResult {
                        new_item,
                        old_item: Some(old_item),
                    }
                } else {
                    unreachable!();
                }
            } else {
                MultiTypeDictInsertResult {
                    new_item,
                    old_item: None,
                }
            }
        } else {
            unreachable!();
        }
    }

    pub fn insert_any(
        &mut self,
        item: impl Any,
        type_id: TypeId,
    ) -> MultiTypeDictInsertResult<dyn Any> {
        let new_item: MultiTypeDictItem<dyn Any> = MultiTypeDictItem {
            type_id: type_id,
            item: Arc::new(item),
        };

        let old_item = self.storage.insert(type_id, new_item.clone());

        MultiTypeDictInsertResult { new_item, old_item }
    }

    pub fn get_item_ref<ItemType>(&self) -> Option<MultiTypeDictItem<ItemType>>
    where
        ItemType: Any,
    {
        let type_id = TypeId::of::<ItemType>();

        self.get_item_ref_any(type_id)
            .map(|item| item.downcast::<ItemType>())
            .flatten()
    }

    pub fn get_item_ref_any(&self, type_id: TypeId) -> Option<MultiTypeDictItem<dyn Any>> {
        self.storage.get(&type_id).map(|item| item.clone())
    }

    pub fn remove<ItemType>(&mut self) -> Option<MultiTypeDictItem<ItemType>>
    where
        ItemType: Any,
    {
        let type_id = TypeId::of::<ItemType>();

        self.remove_by_type_id(type_id)
            .map(|item| item.downcast::<ItemType>())
            .flatten()
    }

    pub fn remove_by_type_id(&mut self, type_id: TypeId) -> Option<MultiTypeDictItem<dyn Any>> {
        self.storage.remove(&type_id)
    }

    pub fn iter(&self) -> MultiTypeDictIterator {
        MultiTypeDictIterator {
            inner_iterator: self.storage.iter(),
        }
    }
}

impl MultiTypeDictItem<dyn Any> {
    pub fn downcast<CastType: 'static>(&self) -> Option<MultiTypeDictItem<CastType>> {
        if self.item.is::<CastType>() {
            let ptr = Arc::into_raw(self.item.clone()).cast::<CastType>();

            Some(MultiTypeDictItem {
                type_id: self.type_id,
                item: unsafe { Arc::from_raw(ptr) },
            })
        } else {
            None
        }
    }
}

impl<ItemType: ?Sized> MultiTypeDictItem<ItemType> {
    pub fn as_arc_ref(&self) -> &Arc<ItemType> {
        &self.item
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

#[cfg(test)]
mod tests {
    use std::{any::Any, sync::Arc};

    use crate::muleengine::containers::multi_type_dict::MultiTypeDictItem;

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
            .old_item
            .is_none());

        assert_eq!(
            *dict
                .insert(A {
                    value: "A1".to_string(),
                })
                .old_item
                .unwrap()
                .as_arc_ref(),
            Arc::new(A {
                value: "A0".to_string(),
            })
        );

        assert!(dict
            .insert(B {
                value: "B".to_string(),
            })
            .old_item
            .is_none());

        assert_eq!(
            *dict.get_item_ref::<A>().unwrap().as_arc_ref(),
            Arc::new(A {
                value: "A1".to_string(),
            })
        );

        assert_eq!(
            *dict.get_item_ref::<B>().unwrap().as_arc_ref(),
            Arc::new(B {
                value: "B".to_string(),
            })
        );

        let systems: Vec<MultiTypeDictItem<dyn Any>> = dict.iter().collect();
        assert_eq!(systems.len(), 2);
        if let Some(_) = systems[0].downcast::<A>() {
            assert!(systems[0].downcast::<A>().is_some());
            assert!(systems[1].downcast::<B>().is_some());
        } else {
            assert!(systems[0].downcast::<B>().is_some());
            assert!(systems[1].downcast::<A>().is_some());
        }

        assert_eq!(
            *dict.remove::<A>().unwrap().as_arc_ref(),
            Arc::new(A {
                value: "A1".to_string(),
            })
        );

        assert!(dict.get_item_ref::<A>().is_none());

        assert_eq!(
            *dict.remove::<B>().unwrap().as_arc_ref(),
            Arc::new(B {
                value: "B".to_string(),
            })
        );

        assert!(dict.get_item_ref::<B>().is_none());
    }
}
