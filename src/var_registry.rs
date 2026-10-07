//! The solver's variables: every entity's own numbers (a point's x and y, a
//! radius, an arc's angles, an ellipse's `radmin`) laid out in one global
//! vector, with which ones are fixed. [`VarEntity`] is how an entity exposes
//! its numbers; [`VarRegistry`] holds the vector.

use std::collections::HashMap;

/// Trait for geometric entities that can provide variables to the solver
pub trait VarEntity {
    /// Get the current variable values as a vector
    fn values(&self) -> Vec<f64>;

    /// Update the entity with new variable values
    fn set_values(&mut self, params: &[f64]) -> Result<(), String>;

    /// Get the names/descriptions of each variable (for debugging)
    fn var_names(&self) -> Vec<String>;

    /// Get the number of variables this entity exposes
    fn num_vars(&self) -> usize {
        self.values().len()
    }

    /// Check if a variable is fixed (shouldn't be modified by solver)
    fn is_var_fixed(&self, index_in_entity: usize) -> bool;
}

/// Information about a variable in the global variable vector
#[derive(Debug, Clone)]
pub struct VarInfo {
    /// ID of the entity that owns it.
    pub entity_id: String,
    /// Kind of that entity.
    pub entity_type: EntityType,
    /// Its index among the entity's values ([`VarEntity::values`]).
    pub index_in_entity: usize,
    /// Its index in the global vector.
    pub global_index: usize,
    /// A readable name, such as `p1.x` or `a1.start_angle`.
    pub name: String,
    /// Whether the solver must hold it.
    pub is_fixed: bool,
}

/// The kinds of entity that own variables (a Line owns none).
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    /// `x`, `y`.
    Point,
    /// `radius`.
    Circle,
    /// `radius`, `start_angle`, `end_angle`.
    Arc,
    /// `radmin`.
    Ellipse,
    /// `radmin`, `start_angle`, `end_angle`.
    EllipticalArc,
}

/// Manages the global variable vector and entity-to-variable mapping
pub struct VarRegistry {
    /// Maps entity ID to its starting index in the global variable vector
    entity_to_global_index: HashMap<String, usize>,

    /// Maps entity ID to its type
    entity_types: HashMap<String, EntityType>,

    /// Complete information about each variable
    info: Vec<VarInfo>,

    /// Current global variable vector
    values: Vec<f64>,
}

impl Default for VarRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl VarRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self {
            entity_to_global_index: HashMap::new(),
            entity_types: HashMap::new(),
            info: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Register an entity with the variable manager
    pub fn register_entity<T: VarEntity>(
        &mut self,
        entity_id: String,
        entity_type: EntityType,
        entity: &T,
    ) {
        let start_index = self.values.len();
        let entity_params = entity.values();
        let param_names = entity.var_names();

        // Add entity to mapping
        self.entity_to_global_index
            .insert(entity_id.clone(), start_index);
        self.entity_types
            .insert(entity_id.clone(), entity_type.clone());

        // Add variables to global vector
        self.values.extend(entity_params.iter());

        // Add variable info
        for (param_idx, (_param_value, param_name)) in
            entity_params.iter().zip(param_names.iter()).enumerate()
        {
            let global_idx = start_index + param_idx;
            let is_fixed = entity.is_var_fixed(param_idx);

            self.info.push(VarInfo {
                entity_id: entity_id.clone(),
                entity_type: entity_type.clone(),
                index_in_entity: param_idx,
                global_index: global_idx,
                name: param_name.clone(),
                is_fixed,
            });
        }
    }

    /// Get the global variable index for a specific entity variable
    pub fn get_global_index(&self, entity_id: &str, index_in_entity: usize) -> Option<usize> {
        self.entity_to_global_index
            .get(entity_id)
            .map(|&start_idx| start_idx + index_in_entity)
    }


    /// Get the current global variable vector
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Get a mutable reference to the global variable vector
    pub fn values_mut(&mut self) -> &mut [f64] {
        &mut self.values
    }

    /// Update a specific variable
    pub fn set_value(&mut self, global_index: usize, value: f64) -> Result<(), String> {
        if global_index >= self.values.len() {
            return Err("variable index out of bounds".to_string());
        }

        // Check if variable is fixed
        if let Some(info) = self.info.get(global_index)
            && info.is_fixed
        {
            return Err(format!(
                "variable {} is fixed and cannot be modified",
                info.name
            ));
        }

        self.values[global_index] = value;
        Ok(())
    }

    /// Update variables for a specific entity
    pub fn write_entity_values<T: VarEntity>(
        &mut self,
        entity_id: &str,
        entity: &mut T,
    ) -> Result<(), String> {
        if let Some(&start_idx) = self.entity_to_global_index.get(entity_id) {
            let param_count = entity.num_vars();
            let entity_params = &self.values[start_idx..start_idx + param_count];
            entity.set_values(entity_params)
        } else {
            Err(format!(
                "entity {} not found in the variable registry",
                entity_id
            ))
        }
    }


    /// Get total number of variables
    pub fn num_vars(&self) -> usize {
        self.values.len()
    }

    /// Get variable info for debugging
    pub fn var_info(&self) -> &[VarInfo] {
        &self.info
    }


}
