I think I'm done w/ all these below, can be deleted:-------

impl Controller {

// Trait definitions for the data holders
trait AttributeDataHolder: Clone {
    fn get_attr_type_id(&self) -> i64;
    fn set_attr_type_id(&mut self, id: i64);
    fn as_relation_to_entity_data_holder(&self) -> Option<&RelationToEntityDataHolder> { None }
    fn as_relation_to_entity_data_holder_mut(&mut self) -> Option<&mut RelationToEntityDataHolder> { None }
    fn as_attribute_with_valid_and_observed_dates(&self) -> Option<&dyn AttributeWithValidAndObservedDates> { None }
    fn as_attribute_with_valid_and_observed_dates_mut(&mut self) -> Option<&mut dyn AttributeWithValidAndObservedDates> { None }
}

trait AttributeWithValidAndObservedDates {
    fn get_valid_on_date(&self) -> Option<i64>;
    fn get_observation_date(&self) -> i64;
    fn set_valid_on_date(&mut self, date: Option<i64>);
    fn set_observation_date(&mut self, date: i64);
}

// Implement traits for data holders
impl AttributeDataHolder for QuantityAttributeDataHolder {
    fn get_attr_type_id(&self) -> i64 { self.attr_type_id }
    fn set_attr_type_id(&mut self, id: i64) { self.attr_type_id = id; }
    fn as_attribute_with_valid_and_observed_dates(&self) -> Option<&dyn AttributeWithValidAndObservedDates> {
        Some(self)
    }
    fn as_attribute_with_valid_and_observed_dates_mut(&mut self) -> Option<&mut dyn AttributeWithValidAndObservedDates> {
        Some(self)
    }
}

impl AttributeWithValidAndObservedDates for QuantityAttributeDataHolder {
    fn get_valid_on_date(&self) -> Option<i64> { self.valid_on_date }
    fn get_observation_date(&self) -> i64 { self.observation_date }
    fn set_valid_on_date(&mut self, date: Option<i64>) { self.valid_on_date = date; }
    fn set_observation_date(&mut self, date: i64) { self.observation_date = date; }
}

impl AttributeDataHolder for TextAttributeDataHolder {
    fn get_attr_type_id(&self) -> i64 { self.attr_type_id }
    fn set_attr_type_id(&mut self, id: i64) { self.attr_type_id = id; }
    fn as_attribute_with_valid_and_observed_dates(&self) -> Option<&dyn AttributeWithValidAndObservedDates> {
        Some(self)
    }
    fn as_attribute_with_valid_and_observed_dates_mut(&mut self) -> Option<&mut dyn AttributeWithValidAndObservedDates> {
        Some(self)
    }
}

impl AttributeWithValidAndObservedDates for TextAttributeDataHolder {
    fn get_valid_on_date(&self) -> Option<i64> { self.valid_on_date }
    fn get_observation_date(&self) -> i64 { self.observation_date }
    fn set_valid_on_date(&mut self, date: Option<i64>) { self.valid_on_date = date; }
    fn set_observation_date(&mut self, date: i64) { self.observation_date = date; }
}

impl AttributeDataHolder for DateAttributeDataHolder {
    fn get_attr_type_id(&self) -> i64 { self.attr_type_id }
    fn set_attr_type_id(&mut self, id: i64) { self.attr_type_id = id; }
}

impl AttributeDataHolder for BooleanAttributeDataHolder {
    fn get_attr_type_id(&self) -> i64 { self.attr_type_id }
    fn set_attr_type_id(&mut self, id: i64) { self.attr_type_id = id; }
    fn as_attribute_with_valid_and_observed_dates(&self) -> Option<&dyn AttributeWithValidAndObservedDates> {
        Some(self)
    }
    fn as_attribute_with_valid_and_observed_dates_mut(&mut self) -> Option<&mut dyn AttributeWithValidAndObservedDates> {
        Some(self)
    }
}

impl AttributeWithValidAndObservedDates for BooleanAttributeDataHolder {
    fn get_valid_on_date(&self) -> Option<i64> { self.valid_on_date }
    fn get_observation_date(&self) -> i64 { self.observation_date }
    fn set_valid_on_date(&mut self, date: Option<i64>) { self.valid_on_date = date; }
    fn set_observation_date(&mut self, date: i64) { self.observation_date = date; }
}

impl AttributeDataHolder for FileAttributeDataHolder {
    fn get_attr_type_id(&self) -> i64 { self.attr_type_id }
    fn set_attr_type_id(&mut self, id: i64) { self.attr_type_id = id; }
}

impl AttributeDataHolder for RelationToEntityDataHolder {
    fn get_attr_type_id(&self) -> i64 { self.attr_type_id }
    fn set_attr_type_id(&mut self, id: i64) { self.attr_type_id = id; }
    fn as_relation_to_entity_data_holder(&self) -> Option<&RelationToEntityDataHolder> { Some(self) }
    fn as_relation_to_entity_data_holder_mut(&mut self) -> Option<&mut RelationToEntityDataHolder> { Some(self) }
    fn as_attribute_with_valid_and_observed_dates(&self) -> Option<&dyn AttributeWithValidAndObservedDates> {
        Some(self)
    }
    fn as_attribute_with_valid_and_observed_dates_mut(&mut self) -> Option<&mut dyn AttributeWithValidAndObservedDates> {
        Some(self)
    }
}

impl AttributeWithValidAndObservedDates for RelationToEntityDataHolder {
    fn get_valid_on_date(&self) -> Option<i64> { self.valid_on_date }
    fn get_observation_date(&self) -> i64 { self.observation_date }
    fn set_valid_on_date(&mut self, date: Option<i64>) { self.valid_on_date = date; }
    fn set_observation_date(&mut self, date: i64) { self.observation_date = date; }
}

impl AttributeDataHolder for RelationToGroupDataHolder {
    fn get_attr_type_id(&self) -> i64 { self.attr_type_id }
    fn set_attr_type_id(&mut self, id: i64) { self.attr_type_id = id; }
    fn as_attribute_with_valid_and_observed_dates(&self) -> Option<&dyn AttributeWithValidAndObservedDates> {
        Some(self)
    }
    fn as_attribute_with_valid_and_observed_dates_mut(&mut self) -> Option<&mut dyn AttributeWithValidAndObservedDates> {
        Some(self)
    }
}

impl AttributeWithValidAndObservedDates for RelationToGroupDataHolder {
    fn get_valid_on_date(&self) -> Option<i64> { self.valid_on_date }
    fn get_observation_date(&self) -> i64 { self.observation_date }
    fn set_valid_on_date(&mut self, date: Option<i64>) { self.valid_on_date = date; }
    fn set_observation_date(&mut self, date: i64) { self.observation_date = date; }
}

