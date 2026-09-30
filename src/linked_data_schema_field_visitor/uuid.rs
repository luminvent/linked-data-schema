use crate::LinkedDataSchemaFieldVisitor;
use iri_s::iri;
use prefixmap::IriRef;
use shacl::ast::ASTComponent;

impl LinkedDataSchemaFieldVisitor for ::uuid::Uuid {
  fn value_components() -> Vec<ASTComponent> {
    vec![
      ASTComponent::Datatype(IriRef::iri(iri!("http://www.w3.org/2001/XMLSchema#string"))),
      ASTComponent::Pattern {
        pattern:
          "^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-5][0-9a-f]{3}-[089ab][0-9a-f]{3}-[0-9a-f]{12}$"
            .to_string(),
        flags: Some("i".to_string()),
      },
    ]
  }
}
