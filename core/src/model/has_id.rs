/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010-2017 inclusive, 2020, and 2023-2025 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/

use crate::model::entity::Entity;
use crate::model::entity_class::EntityClass;
use crate::model::group::Group;
use crate::model::relation_type::RelationType;
use crate::model::om_instance::OmInstance;
//use std::any::Any;

/// Trait for objects with IDs. See usus for example in controller.rs.
pub trait HasId { //%%?:  : std::any::Any {
    fn get_id(&self) -> i64;
    //%% these and others below?:
    //fn get_class_name(&self) -> &str;
    //fn as_any(&self) -> &dyn std::any::Any;
}

//%%should this be moved to entity.rs, and others below similarly to other files?
impl HasId for Entity {
    fn get_id(&self) -> i64 { self.get_id() }
    //fn get_class_name(&self) -> &str { "Entity" }
    //fn as_any(&self) -> &dyn std::any::Any { self }
}

impl HasId for Group {
    fn get_id(&self) -> i64 { self.get_id() }
    //fn get_class_name(&self) -> &str { "Group" }
    //fn as_any(&self) -> &dyn std::any::Any { self }
}

impl HasId for EntityClass {
    fn get_id(&self) -> i64 { self.get_id() }
    //fn get_class_name(&self) -> &str { "EntityClass" }
    //fn as_any(&self) -> &dyn std::any::Any { self }
}

impl HasId for RelationType {
    fn get_id(&self) -> i64 { self.get_id() }
    //fn get_class_name(&self) -> &str { "RelationType" }
    //fn as_any(&self) -> &dyn std::any::Any { self }
}

impl HasId for OmInstance {
    fn get_id(&self) -> i64 { 0 } // OmInstance uses String IDs
    //fn get_class_name(&self) -> &str { "OmInstance" }
    //fn as_any(&self) -> &dyn std::any::Any { self }
}

//%%pub trait HasIdAny: HasId + std::any::Any {}
//%%... impl Any for Entity {}
//impl Any for Group {}
//impl Any for EntityClass {}
//impl Any for RelationType {}
//impl Any for OmInstance {}
