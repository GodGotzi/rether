use crate::{
    alloc::BufferAllocationID,
    model::{BufferLocation, Expandable, Model},
    Geometry, ModelContext, Rotate, Scale, Transform, Translate,
};

use super::handle::TreeHandle;

#[derive(Debug, Clone)]
pub enum TreeModel<T, C> {
    Root {
        geometry: Geometry<T>,
        sub_models: Vec<TreeModel<T, C>>,
        ctx: C,
    },
    Node {
        location: BufferLocation,
        sub_models: Vec<TreeModel<T, C>>,
        ctx: C,
    },
}

impl<T: Clone, C: Translate + Rotate + Scale> Model<T, TreeHandle<C>> for TreeModel<T, C> {
    fn geometry(&self) -> &Geometry<T> {
        match self {
            Self::Root { geometry, .. } => geometry,
            Self::Node { .. } => panic!("Cannot get geometry from node"),
        }
    }

    fn into_handle(self, allocation_id: BufferAllocationID) -> TreeHandle<C> {
        match self {
            Self::Root {
                sub_models, ctx, ..
            } => TreeHandle::Root {
                id: allocation_id.clone(),
                transform: Transform::default(),
                sub_handles: sub_models
                    .into_iter()
                    .map(|model| model.into_handle(allocation_id.clone()))
                    .collect(),
                ctx,
            },
            Self::Node {
                location,
                sub_models,
                ctx,
            } => TreeHandle::Node {
                location: location.clone(),
                sub_handles: sub_models
                    .into_iter()
                    .map(|model| model.into_handle(allocation_id.clone()))
                    .collect(),
                ctx,
            },
        }
    }
}

impl<T: Clone, C: ModelContext> TreeModel<T, C> {
    pub fn expand(&mut self, model: TreeModel<T, C>) {
        let (node, node_geometry) = model.into_node(0);

        match self {
            Self::Root {
                sub_models,
                ctx,
                geometry,
                ..
            } => {
                geometry.expand(&node_geometry);
                ctx.expand(node.ctx());
                sub_models.push(node);
            }
            Self::Node {
                sub_models, ctx, ..
            } => {
                ctx.expand(node.ctx());
                sub_models.push(node);
            }
        }
    }

    pub fn ctx(&self) -> &C {
        match self {
            Self::Root { ctx, .. } => ctx,
            Self::Node { ctx, .. } => ctx,
        }
    }

    pub fn into_node(self, offset: usize) -> (TreeModel<T, C>, Geometry<T>) {
        match self {
            Self::Root {
                geometry,
                sub_models,
                ctx,
            } => (
                TreeModel::Node {
                    location: BufferLocation {
                        offset,
                        size: geometry.vertices().len(),
                    },
                    sub_models,
                    ctx,
                },
                geometry,
            ),
            Self::Node { .. } => panic!("Cannot convert node to node"),
        }
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable + Clone + Translate, C: Translate> Translate
    for TreeModel<T, C>
{
    fn translate(&mut self, translation: glam::Vec3) {
        match self {
            Self::Root {
                geometry,
                sub_models,
                ctx,
                ..
            } => {
                ctx.translate(translation);
                geometry.translate(translation);
                for model in sub_models.iter_mut() {
                    model.translate(translation);
                }
            }
            Self::Node {
                ctx, sub_models, ..
            } => {
                ctx.translate(translation);
                for model in sub_models.iter_mut() {
                    model.translate(translation);
                }
            }
        }
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable + Clone + Rotate, C: Rotate> Rotate for TreeModel<T, C> {
    fn rotate(&mut self, rotation: glam::Quat) {
        match self {
            Self::Root {
                geometry,
                sub_models,
                ctx,
                ..
            } => {
                geometry.rotate(rotation);
                ctx.rotate(rotation);
                for model in sub_models.iter_mut() {
                    model.rotate(rotation);
                }
            }
            Self::Node {
                ctx, sub_models, ..
            } => {
                ctx.rotate(rotation);
                for model in sub_models.iter_mut() {
                    model.rotate(rotation);
                }
            }
        }
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable + Clone + Scale, C: Scale> Scale for TreeModel<T, C> {
    fn scale(&mut self, scale: glam::Vec3) {
        match self {
            Self::Root {
                geometry,
                sub_models,
                ctx,
                ..
            } => {
                geometry.scale(scale);
                ctx.scale(scale);
                for model in sub_models.iter_mut() {
                    model.scale(scale);
                }
            }
            Self::Node {
                ctx, sub_models, ..
            } => {
                ctx.scale(scale);
                for model in sub_models.iter_mut() {
                    model.scale(scale);
                }
            }
        }
    }
}
