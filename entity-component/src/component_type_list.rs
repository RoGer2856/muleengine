use std::any::TypeId;

#[macro_export]
macro_rules! component_type_list {
    ($type0:ty $(, $types:ty)* $(,)?) => {
        [
            std::any::TypeId::of::<$type0>(),
            $(std::any::TypeId::of::<$types>(),)*
        ]
    };
}

pub trait ToSortedComponentTypeList {
    fn to_sorted_component_type_list(self) -> Vec<TypeId>;
}

impl<T: AsRef<[TypeId]>> ToSortedComponentTypeList for T {
    fn to_sorted_component_type_list(self) -> Vec<TypeId> {
        let mut type_list = self.as_ref().to_vec();

        type_list.sort_unstable_by(|a, b| (*a).cmp(b));
        type_list.dedup_by(|a, b| *a == *b);

        type_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position(String);
    struct Velocity(String);
    struct Orientation(String);

    #[test]
    fn remove_duplicates() {
        let list = component_type_list![Position, Velocity, Orientation, Velocity,];

        let sorted_list = list.to_sorted_component_type_list();
        assert_eq!(3, sorted_list.len());

        let mut found_position = false;
        let mut found_velocity = false;
        let mut found_orientation = false;
        for type_id in sorted_list {
            if type_id == std::any::TypeId::of::<Position>() {
                found_position = true;
            } else if type_id == std::any::TypeId::of::<Velocity>() {
                found_velocity = true;
            } else if type_id == std::any::TypeId::of::<Orientation>() {
                found_orientation = true;
            }
        }
        assert!(found_position);
        assert!(found_velocity);
        assert!(found_orientation);
    }
}
