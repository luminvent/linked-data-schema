use crate::LinkedDataSchemaFieldVisitor;
use iri_s::iri;
use prefixmap::IriRef;
use shacl::ast::ASTComponent;

impl LinkedDataSchemaFieldVisitor for ::uuid::Uuid {
  fn field_components() -> Vec<ASTComponent> {
    vec![
      ASTComponent::MinCount(1),
      ASTComponent::MaxCount(1),
      ASTComponent::Datatype(Self::type_iri_ref().unwrap()),
      ASTComponent::Pattern {
        pattern:
          "/^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-5][0-9a-f]{3}-[089ab][0-9a-f]{3}-[0-9a-f]{12}$/i"
            .to_string(),
        flags: None,
      },
    ]
  }

  fn type_iri_ref() -> Option<IriRef> {
    Some(IriRef::iri(iri!("xsd::string")))
  }
}
