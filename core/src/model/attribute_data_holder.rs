/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive and 2023-2025 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
//use super::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use anyhow::anyhow;

#[derive(Debug)]
pub enum AttributeDataHolder {
    //%%was in Scala:  extends AttributeDHWithVODates(rel_type_id_in, valid_on_date_in, observation_date_in)
    //AttributeDHWithVODates {
    //    attr_type_id: i64,
    //    valid_on_date: Option<i64>,
    //    observation_date: i64,
    //},
    QuantityAttributeDH { qadh: QuantityAttributeDH },
    TextAttributeDH { tadh: TextAttributeDH },
    RelationToEntityDH { rtedh: RelationToEntityDH },
    GroupDH { gdh: GroupDH },
    RelationToGroupDH { rtgdh: RelationToGroupDH },
    DateAttributeDH { dadh: DateAttributeDH },
    BooleanAttributeDH { badh: BooleanAttributeDH },
    FileAttributeDH { fadh: FileAttributeDH },
}

#[derive(Debug)]
pub struct QuantityAttributeDH {
    pub attr_type_id: i64,
    pub valid_on_date: Option<i64>,
    pub observation_date: i64,
    pub number: f64,
    pub unit_id: i64,
}
#[derive(Debug)]
pub struct TextAttributeDH {
    pub attr_type_id: i64,
    pub valid_on_date: Option<i64>,
    pub observation_date: i64,
    pub text: String,
}
#[derive(Debug)]
pub struct RelationToEntityDH {
    pub rel_type_id: i64,
    pub valid_on_date: Option<i64>,
    pub observation_date: i64,
    pub entity_id2: i64,
    pub is_remote: bool,
    pub remote_instance_id: String,
}
#[derive(Debug)]
pub struct GroupDH {
    pub id: i64,
    pub name: String,
    pub insertion_date: Option<i64>,
    pub mixed_classes_allowed: bool,
}
#[derive(Debug)]
pub struct RelationToGroupDH {
    pub entity_id: i64,
    pub rel_type_id: i64,
    pub group_id: i64,
    pub valid_on_date: Option<i64>,
    pub observation_date: i64,
}
#[derive(Debug)]
pub struct DateAttributeDH {
    pub attr_type_id: i64,
    pub date: i64,
}
#[derive(Debug)]
pub struct BooleanAttributeDH {
    pub attr_type_id: i64,
    pub valid_on_date: Option<i64>,
    pub observation_date: i64,
    pub boolean: bool,
}
#[derive(Debug)]
pub struct FileAttributeDH {
    pub attr_type_id: i64,
    pub description: String,
    pub original_file_path: String,
}

impl AttributeDataHolder {
    pub fn get_attr_type_id(&self) -> Result<i64, anyhow::Error> {
        let id: i64 = match self {
            AttributeDataHolder::QuantityAttributeDH { qadh } => qadh.attr_type_id,
            AttributeDataHolder::TextAttributeDH { tadh } => tadh.attr_type_id,
            AttributeDataHolder::DateAttributeDH { dadh } => dadh.attr_type_id,
            AttributeDataHolder::BooleanAttributeDH { badh } => badh.attr_type_id,
            AttributeDataHolder::FileAttributeDH { fadh } => fadh.attr_type_id,
            AttributeDataHolder::RelationToEntityDH { rtedh } => rtedh.rel_type_id,
            AttributeDataHolder::RelationToGroupDH { rtgdh } => rtgdh.rel_type_id,
            AttributeDataHolder::GroupDH { .. } => {
                return Err(anyhow!(
                    "get_attr_type_id doesn't apply to AttributeDataHolder::GroupDH"
                ));
            } //%%x => return Err(anyhow!("unexpected value: {:?}", x)),
        };
        Ok(id)
    }

    pub fn set_attr_type_id(&mut self, id: i64) -> Result<(), anyhow::Error> {
        match self {
            AttributeDataHolder::QuantityAttributeDH { qadh } => qadh.attr_type_id = id,
            AttributeDataHolder::TextAttributeDH { tadh } => tadh.attr_type_id = id,
            AttributeDataHolder::DateAttributeDH { dadh } => dadh.attr_type_id = id,
            AttributeDataHolder::BooleanAttributeDH { badh } => badh.attr_type_id = id,
            AttributeDataHolder::FileAttributeDH { fadh } => fadh.attr_type_id = id,
            AttributeDataHolder::RelationToEntityDH { rtedh } => rtedh.rel_type_id = id,
            AttributeDataHolder::RelationToGroupDH { rtgdh } => rtgdh.rel_type_id = id,
            AttributeDataHolder::GroupDH { .. } => {
                return Err(anyhow!(
                    "get_attr_type_id doesn't apply to AttributeDataHolder:;GroupDH"
                ));
            } //%%x => return Err(anyhow!("unexpected value: {:?}", x)),
        };
        Ok(())
    }
    
    pub fn as_valid_and_observation_dates_data_holder_mut(&mut self) -> Option<&mut dyn AttributeDataHolderWithVODates> {
        match self {
            AttributeDataHolder::QuantityAttributeDH { qadh } => Some(qadh),
            AttributeDataHolder::TextAttributeDH { tadh } => Some(tadh),
            AttributeDataHolder::RelationToEntityDH { rtedh } => Some(rtedh),
            AttributeDataHolder::RelationToGroupDH { rtgdh } => Some(rtgdh),
            AttributeDataHolder::BooleanAttributeDH { .. } => None,
            AttributeDataHolder::DateAttributeDH { .. } => None,
            AttributeDataHolder::FileAttributeDH { .. } => None,
            AttributeDataHolder::GroupDH { .. } => None,
        }
    }
    
    pub fn as_relation_to_entity_data_holder_mut(&mut self) -> Option<&mut RelationToEntityDH> {
        match self {
            AttributeDataHolder::RelationToEntityDH { rtedh } => Some(rtedh),
            _ => None,
        }
    }
}

pub trait AttributeDataHolderWithVODates {
    fn get_valid_on_date(&mut self) -> Option<i64> ;
    fn set_valid_on_date(&mut self, date: Option<i64>);
    fn get_observation_date(&mut self) -> i64;
    fn set_observation_date(&mut self, date: i64);
}
impl AttributeDataHolderWithVODates for BooleanAttributeDH {
    fn get_valid_on_date(&mut self) -> Option<i64> {
        self.valid_on_date
    }
    fn set_valid_on_date(&mut self, date: Option<i64>) {
        self.valid_on_date = date;
    }
    fn get_observation_date(&mut self) -> i64 {
        self.observation_date
    }
    fn set_observation_date(&mut self, date: i64) {
        self.observation_date = date;
    }
}
impl AttributeDataHolderWithVODates for QuantityAttributeDH {
    fn get_valid_on_date(&mut self) -> Option<i64> {
        self.valid_on_date
    }
    fn set_valid_on_date(&mut self, date: Option<i64>) {
        self.valid_on_date = date;
    }
    fn get_observation_date(&mut self) -> i64 {
        self.observation_date
    }
    fn set_observation_date(&mut self, date: i64) {
        self.observation_date = date;
    }
}
impl AttributeDataHolderWithVODates for TextAttributeDH {
    fn get_valid_on_date(&mut self) -> Option<i64> {
        self.valid_on_date
    }
    fn set_valid_on_date(&mut self, date: Option<i64>) {
        self.valid_on_date = date;
    }
    fn get_observation_date(&mut self) -> i64 {
        self.observation_date
    }
    fn set_observation_date(&mut self, date: i64) {
        self.observation_date = date;
    }
}
impl AttributeDataHolderWithVODates for RelationToEntityDH {
    fn get_valid_on_date(&mut self) -> Option<i64> {
        self.valid_on_date
    }
    fn set_valid_on_date(&mut self, date: Option<i64>) {
        self.valid_on_date = date;
    }
    fn get_observation_date(&mut self) -> i64 {
        self.observation_date
    }
    fn set_observation_date(&mut self, date: i64) {
        self.observation_date = date;
    }
}
impl AttributeDataHolderWithVODates for RelationToGroupDH {
    fn get_valid_on_date(&mut self) -> Option<i64> {
        self.valid_on_date
    }
    fn set_valid_on_date(&mut self, date: Option<i64>) {
        self.valid_on_date = date;
    }
    fn get_observation_date(&mut self) -> i64 {
        self.observation_date
    }
    fn set_observation_date(&mut self, date: i64) {
        self.observation_date = date;
    }
}
//%%was in Scala:  extends AttributeDHWithVODates(rel_type_id_in, valid_on_date_in, observation_date_in)
