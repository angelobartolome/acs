use std::collections::HashMap;

/// Trait for geometric entities that can provide parameters to the solver
pub trait ParametricEntity {
    /// Get the current parameter values as a vector
    fn get_parameters(&self) -> Vec<f64>;

    /// Update the entity with new parameter values
    fn set_parameters(&mut self, params: &[f64]) -> Result<(), String>;

    /// Get the names/descriptions of each parameter (for debugging)
    fn parameter_names(&self) -> Vec<String>;

    /// Get the number of parameters this entity exposes
    fn num_parameters(&self) -> usize {
        self.get_parameters().len()
    }

    /// Check if a parameter is fixed (shouldn't be modified by solver)
    fn is_parameter_fixed(&self, param_index: usize) -> bool;
}

/// Information about a parameter in the global parameter vector
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub param_index: usize,  // Index within the entity's parameter vector
    pub global_index: usize, // Index in the global parameter vector
    pub name: String,
    pub is_fixed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Point,
    Circle,
    Arc,
    // Add more entity types as needed
}

/// Manages the global parameter vector and entity-to-parameter mapping
pub struct ParameterManager {
    /// Maps entity ID to its starting index in the global parameter vector
    entity_to_global_index: HashMap<String, usize>,

    /// Maps entity ID to its type
    entity_types: HashMap<String, EntityType>,

    /// Complete information about each parameter
    parameter_info: Vec<ParameterInfo>,

    /// Current global parameter vector
    parameters: Vec<f64>,
}

impl Default for ParameterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterManager {
    pub fn new() -> Self {
        Self {
            entity_to_global_index: HashMap::new(),
            entity_types: HashMap::new(),
            parameter_info: Vec::new(),
            parameters: Vec::new(),
        }
    }

    /// Register an entity with the parameter manager
    pub fn register_entity<T: ParametricEntity>(
        &mut self,
        entity_id: String,
        entity_type: EntityType,
        entity: &T,
    ) {
        let start_index = self.parameters.len();
        let entity_params = entity.get_parameters();
        let param_names = entity.parameter_names();

        // Add entity to mapping
        self.entity_to_global_index
            .insert(entity_id.clone(), start_index);
        self.entity_types
            .insert(entity_id.clone(), entity_type.clone());

        // Add parameters to global vector
        self.parameters.extend(entity_params.iter());

        // Add parameter info
        for (param_idx, (_param_value, param_name)) in
            entity_params.iter().zip(param_names.iter()).enumerate()
        {
            let global_idx = start_index + param_idx;
            let is_fixed = entity.is_parameter_fixed(param_idx);

            self.parameter_info.push(ParameterInfo {
                entity_id: entity_id.clone(),
                entity_type: entity_type.clone(),
                param_index: param_idx,
                global_index: global_idx,
                name: param_name.clone(),
                is_fixed,
            });
        }
    }

    /// Get the global parameter index for a specific entity parameter
    pub fn get_global_index(&self, entity_id: &str, param_index: usize) -> Option<usize> {
        self.entity_to_global_index
            .get(entity_id)
            .map(|&start_idx| start_idx + param_index)
    }

    /// Get all parameter indices for an entity
    pub fn get_entity_indices(&self, entity_id: &str) -> Option<Vec<usize>> {
        if let Some(&start_idx) = self.entity_to_global_index.get(entity_id) {
            // Find the entity type to determine how many parameters it has
            if let Some(_entity_type) = self.entity_types.get(entity_id) {
                let param_count = self
                    .parameter_info
                    .iter()
                    .filter(|info| info.entity_id == entity_id)
                    .count();

                Some((start_idx..start_idx + param_count).collect())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the current global parameter vector
    pub fn get_parameters(&self) -> &[f64] {
        &self.parameters
    }

    /// Get a mutable reference to the global parameter vector
    pub fn get_parameters_mut(&mut self) -> &mut [f64] {
        &mut self.parameters
    }

    /// Update a specific parameter
    pub fn set_parameter(&mut self, global_index: usize, value: f64) -> Result<(), String> {
        if global_index >= self.parameters.len() {
            return Err("Parameter index out of bounds".to_string());
        }

        // Check if parameter is fixed
        if let Some(info) = self.parameter_info.get(global_index) {
            if info.is_fixed {
                return Err(format!(
                    "Parameter {} is fixed and cannot be modified",
                    info.name
                ));
            }
        }

        self.parameters[global_index] = value;
        Ok(())
    }

    /// Update parameters for a specific entity
    pub fn update_entity_parameters<T: ParametricEntity>(
        &mut self,
        entity_id: &str,
        entity: &mut T,
    ) -> Result<(), String> {
        if let Some(&start_idx) = self.entity_to_global_index.get(entity_id) {
            let param_count = entity.num_parameters();
            let entity_params = &self.parameters[start_idx..start_idx + param_count];
            entity.set_parameters(entity_params)
        } else {
            Err(format!(
                "Entity {entity_id} not found in parameter manager"
            ))
        }
    }

    /// Synchronize parameters from entities to the global vector
    pub fn sync_from_entity<T: ParametricEntity>(
        &mut self,
        entity_id: &str,
        entity: &T,
    ) -> Result<(), String> {
        if let Some(&start_idx) = self.entity_to_global_index.get(entity_id) {
            let entity_params = entity.get_parameters();
            for (i, &value) in entity_params.iter().enumerate() {
                self.parameters[start_idx + i] = value;
            }
            Ok(())
        } else {
            Err(format!(
                "Entity {entity_id} not found in parameter manager"
            ))
        }
    }

    /// Get total number of parameters
    pub fn num_parameters(&self) -> usize {
        self.parameters.len()
    }

    /// Get parameter info for debugging
    pub fn get_parameter_info(&self) -> &[ParameterInfo] {
        &self.parameter_info
    }

    /// Get parameter info for a specific global index
    pub fn get_parameter_info_by_index(&self, global_index: usize) -> Option<&ParameterInfo> {
        self.parameter_info.get(global_index)
    }
}
