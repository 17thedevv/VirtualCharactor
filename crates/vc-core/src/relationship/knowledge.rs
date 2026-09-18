use serde::{Deserialize, Serialize};

/// Relationship-scoped knowledge and facts remembered specifically about an actor.
///
/// Under Skill 14 (Relationship Engineering), facts learned about User X
/// MUST reside in their respective Relationship or scoped memory, NEVER in global CharacterState.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipKnowledge {
    pub known_facts: Vec<String>,
}

impl RelationshipKnowledge {
    pub fn new() -> Self {
        Self {
            known_facts: Vec::new(),
        }
    }

    pub fn with_facts(facts: Vec<String>) -> Self {
        Self { known_facts: facts }
    }

    /// Add a new learned fact if not already recorded.
    pub fn add_fact(&mut self, fact: impl Into<String>) {
        let f = fact.into().trim().to_string();
        if !f.is_empty() && !self.known_facts.iter().any(|existing| existing.eq_ignore_ascii_case(&f)) {
            self.known_facts.push(f);
        }
    }

    /// Remove a known fact matching the given predicate.
    pub fn remove_fact(&mut self, query: &str) {
        self.known_facts.retain(|f| !f.eq_ignore_ascii_case(query.trim()));
    }

    /// Check if a fact exists.
    pub fn has_fact(&self, query: &str) -> bool {
        self.known_facts.iter().any(|f| f.eq_ignore_ascii_case(query.trim()))
    }

    /// Return reference to all facts.
    pub fn facts(&self) -> &[String] {
        &self.known_facts
    }

    /// Clear all known facts for privacy / forgetfulness requests.
    pub fn clear(&mut self) {
        self.known_facts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_add_and_deduplicate() {
        let mut k = RelationshipKnowledge::new();
        k.add_fact("Likes green tea");
        k.add_fact("Likes green tea");
        k.add_fact("likes GREEN TEA");

        assert_eq!(k.facts().len(), 1);
        assert!(k.has_fact("likes green tea"));

        k.remove_fact("likes green tea");
        assert_eq!(k.facts().len(), 0);
    }
}
