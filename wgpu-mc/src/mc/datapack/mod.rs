use std::collections::HashMap;
use std::path::PathBuf;

use crate::texture::UV;

use cgmath::{Matrix4, Vector2};
use serde_json::Value;
use std::convert::{TryFrom, TryInto};
use crate::mc::resource::{ResourceProvider};
use std::fmt::{Display, Formatter};

#[derive(Debug, Hash, Clone, std::cmp::Eq)]
pub struct NamespacedResource (pub String, pub String);

impl NamespacedResource {

    pub fn append(&self, a: &str) -> Self {
        Self (self.0.clone(), format!("{}{}", self.1, a))
    }

    pub fn prepend(&self, a: &str) -> Self {
        Self (self.0.clone(), format!("{}{}", a, self.1))
    }

}

impl Display for NamespacedResource {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}:{}", self.0, self.1))
    }
}

impl TryFrom<&str> for NamespacedResource {
    type Error = ();

    fn try_from(string: &str) -> Result<Self, Self::Error> {
        // Parse the rest of the namespace
        let mut split = string.split(':').take(2);

        Ok(match (split.next(), split.next()) {
                (Some(ns), Some(id)) => Self (ns.into(), id.into()),
                (Some(id), None) => Self ("minecraft".into(), id.into()),
                _ => return Err(())
            }
        )
    }
}

impl From<(&str, &str)> for NamespacedResource {

    fn from(strings: (&str, &str)) -> Self {
        Self (strings.0.into(), strings.1.into())
    }

}

impl PartialEq for NamespacedResource {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

#[derive(Debug, Clone, Eq, Hash)]
pub enum TextureVariableOrResource {
    Tag(String),
    Resource(NamespacedResource)
}

impl TextureVariableOrResource {
    #[must_use]
    pub fn is_tag(&self) -> bool {
        matches!(self, TextureVariableOrResource::Tag(_))
    }

    pub fn as_resource(&self) -> Option<&NamespacedResource> {
        match self {
            TextureVariableOrResource::Tag(_) => None,
            TextureVariableOrResource::Resource(ref res) => Some(res)
        }
    }

    pub fn as_tag(&self) -> Option<&str> {
        match self {
            TextureVariableOrResource::Tag(string) => Some(string),
            TextureVariableOrResource::Resource(ref res) => None
        }
    }

}

impl Display for TextureVariableOrResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureVariableOrResource::Tag(tag) => f.write_str(&format!("#{}", tag)),
            TextureVariableOrResource::Resource(res) => f.write_str(&format!("{}:{}", res.0, res.1))
        }
    }
}

impl PartialEq for TextureVariableOrResource {
    fn eq(&self, other: &Self) -> bool {
        match self {
            TextureVariableOrResource::Tag(tag) => {
                if let TextureVariableOrResource::Tag(o) = other {
                    o == tag
                } else {
                    false
                }
            }
            TextureVariableOrResource::Resource(nsa) => {
                if let TextureVariableOrResource::Resource(nsb) = other {
                    nsa == nsb
                } else {
                    false
                }
            }
        }
    }
}

impl TryFrom<&str> for TextureVariableOrResource {
    type Error = ();

    fn try_from(string: &str) -> Result<TextureVariableOrResource, Self::Error> {
        // See if tag and remove # if so
        let is_tag = string.starts_with('#');
        let string = if is_tag { &string[1..] } else { string };

        // Parse the rest of the namespace
        let mut split = string.split(':').take(2);

        Ok(if !is_tag {
            match (split.next(), split.next()) {
                (Some(ns), Some(id)) => TextureVariableOrResource::Resource(NamespacedResource (ns.into(), id.into())),
                (Some(id), None) => TextureVariableOrResource::Resource(NamespacedResource ("minecraft".into(), id.into())),
                _ => return Err(())
            }
        } else {
            TextureVariableOrResource::Tag(string.into())
        })
    }
}

#[derive(Debug, Clone)]
pub struct FaceTexture {
    pub uv: UV,
    pub texture: TextureVariableOrResource,
}

#[derive(Debug, Clone)]
pub struct ElementFaces {
    pub up: Option<FaceTexture>,
    pub down: Option<FaceTexture>,
    pub north: Option<FaceTexture>,
    pub east: Option<FaceTexture>,
    pub south: Option<FaceTexture>,
    pub west: Option<FaceTexture>
}

type ElementCorner = (f32, f32, f32);

#[derive(Debug, Clone)]
pub struct Element {
    pub from: ElementCorner,
    pub to: ElementCorner,
    pub face_textures: ElementFaces,
}

///A struct that described a block and how it renders
/// Not a mesh! That would be BlockstateVariantMesh
#[derive(Clone, Debug)]
pub struct BlockModel {
    pub id: NamespacedResource,
    pub parent: Option<NamespacedResource>,
    pub elements: Vec<Element>,
    ///Depending on the camera state, e.g. 3rd or 1st person, the way the block is rendered is changed
    pub display_transforms: HashMap<String, Matrix4<f32>>,
    pub textures: HashMap<String, TextureVariableOrResource>,
}

impl BlockModel {
    fn triplet_from_array(vec: &[Value]) -> Option<ElementCorner> {

        Some(
            (
                vec[0].as_f64()? as f32,
                vec[1].as_f64()? as f32,
                vec[2].as_f64()? as f32
            )
        )
    }

    fn parse_face(
        face: Option<&Value>,
        textures: &HashMap<String, TextureVariableOrResource>
    ) -> Option<FaceTexture> {
        let face = face?.as_object()?;
        println!("{:?} {:?}", face, textures);
        let uv_arr = face.get("uv")?.as_array()?;

        let uv = (
            //TODO: handle UV rotation
            (
                uv_arr[0].as_f64().unwrap() as f32,
                uv_arr[1].as_f64().unwrap() as f32,
            ),
            (
                uv_arr[2].as_f64().unwrap() as f32,
                uv_arr[3].as_f64().unwrap() as f32,
            ),
        );

        let texture: TextureVariableOrResource = face.get("texture")?.as_str()?.try_into().ok()?;

        let texture = match texture {
            TextureVariableOrResource::Tag(ref tag_value) => {
                // println!("{:?} {:?}", face.get("texture"), textures.get(tag_value));
                // println!("{} {:?}", tag_value, textures.get(tag_value));
                textures.get(tag_value)?.clone()
            }
            TextureVariableOrResource::Resource(_) => texture
        };

        Some(FaceTexture {
            uv,
            texture
        })
    }

    fn parse_elements(
        val: Option<&Value>,
        parent: Option<&BlockModel>,
        textures: &HashMap<String, TextureVariableOrResource>,
    ) -> Option<Vec<Element>> {
        match val {
            //No elements, default to parent's elements
            None => match parent {
                Some(parent) => Some(parent.elements.clone()),
                None => Some(Vec::new()),
            },
            Some(v) => {
                v.as_array()?
                    .iter()
                    .map(|element| {
                        let triplet = Self::triplet_from_array(element.get("from")?.as_array()?)?;
                        let from = (triplet.0 / 16.0, triplet.1 / 16.0, triplet.2 / 16.0);

                        let triplet = Self::triplet_from_array(element.get("to")?.as_array()?)?;
                        let to = (triplet.0 / 16.0, triplet.1 / 16.0, triplet.2 / 16.0);

                        let faces = element.get("faces")?.as_object()?;

                        // println!("{:?}", faces);

                        Some(Element {
                            from,
                            to,
                            face_textures: {
                                ElementFaces {
                                    up: Self::parse_face(faces.get("up"), &textures),
                                    down: Self::parse_face(faces.get("down"), &textures),
                                    north: Self::parse_face(faces.get("north"), &textures),
                                    east: Self::parse_face(faces.get("east"), &textures),
                                    south: Self::parse_face(faces.get("south"), &textures),
                                    west: Self::parse_face(faces.get("west"), &textures),
                                }
                            },
                        })
                    }).collect::<Option<Vec<Element>>>()
            }
        }
    }

    pub fn deserialize<'a>(
        identifier: &NamespacedResource,
        resource_provider: &dyn ResourceProvider,
        model_map: &'a mut HashMap<NamespacedResource, BlockModel>,
    ) -> Option<&'a Self> {

        if model_map.contains_key(identifier) {
            return model_map.get(identifier);
        }

        let bytes = resource_provider.get_resource(&identifier.prepend("models/").append(".json"));
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        let obj = json.as_object()?;

        //Get information about the parent model, if this model has one
        let parent = obj.get("parent").and_then(|v| {
            let parent_identifier_string = v.as_str()?;
            let parent_identifier: NamespacedResource = parent_identifier_string.try_into().unwrap();

            BlockModel::deserialize(&parent_identifier, resource_provider, model_map);

            model_map.get(&parent_identifier)
        });

        //Get the face texture mappings
        let mut textures: HashMap<String, TextureVariableOrResource> = obj.get("textures").map_or(
            HashMap::new(),
            |textures_map| {
                //Map of the faces and their textures
                textures_map
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(key, val)| {
                        (
                            key.clone(),
                            val.as_str().unwrap().try_into().unwrap()
                        )
                    })
                    .collect()
            }
        );

        match parent {
            None => {},
            Some(parent_model) => {
                println!("{:?}\n{:?}\n\n", parent_model.textures, textures);

                textures.extend(parent_model.textures.iter()
                    //Map the resolvable tags
                    .map(|(key, value)| {
                        match value {
                            TextureVariableOrResource::Tag(tag_value) => {
                                (
                                    key.clone(),
                                    textures.get(tag_value)
                                        .cloned()
                                        .unwrap_or(value.clone())
                                )
                            }
                            TextureVariableOrResource::Resource(resource) => {
                                (
                                    key.clone(),
                                    TextureVariableOrResource::Resource(resource.clone())
                                )
                            }
                        }
                    }).collect::<Vec<(String, TextureVariableOrResource)>>().into_iter()
                );
            }
        };

        let model = BlockModel {
            id: identifier.clone(),
            parent: parent.map(|some| {
                some.id.clone()
            }),
            elements: {
                Self::parse_elements(obj.get("elements"), parent, &textures)?
            },
            textures,
            display_transforms: HashMap::new(), //TODO
        };

        // println!("{:?}", model);

        model_map.insert(identifier.clone(), model);

        Some(model_map.get(identifier).unwrap())
    }
}
