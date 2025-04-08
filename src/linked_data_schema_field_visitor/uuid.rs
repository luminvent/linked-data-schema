use crate::LinkedDataSchemaFieldVisitor;
use iri_s::iri;
use prefixmap::IriRef;
use shacl_ast::component::Component;

impl LinkedDataSchemaFieldVisitor for ::uuid::Uuid {
  fn field_components() -> Vec<Component> {
    vec![
      Component::MinCount(1),
      Component::MaxCount(1),
      Component::Datatype(Self::type_iri_ref().unwrap()),
      Component::Pattern {
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
