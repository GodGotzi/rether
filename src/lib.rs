mod buffer;
pub mod light;
pub mod model;
pub mod picking;
pub mod texture;
pub mod vertex;

pub use buffer::Buffer;
pub use buffer::IndexedBuffer;

pub use buffer::alloc;

pub use model::geometry::SimpleGeometry;
pub use model::transform::{Rotate, Scale, Transform, Translate};

#[test]
fn test() {
    use buffer::alloc::DynamicAllocHandle;
    use glam::Vec3;
    use model::TreeModel;
    use vertex::Vertex;

    use crate::alloc::BufferAlloc;

    use model::Model;

    use crate::alloc::BufferDynamicAlloc;

    struct TestContext {
        point: Vec3,
    }

    impl Translate for TestContext {
        fn translate(&mut self, translation: Vec3) {
            self.point += translation;
        }
    }

    impl Rotate for TestContext {
        fn rotate(&mut self, _rotation: glam::Quat) {
            todo!()
        }
    }

    impl Scale for TestContext {
        fn scale(&mut self, _scale: Vec3) {
            todo!()
        }
    }

    let gpu_buffer = std::rc::Rc::new(std::cell::RefCell::new(vec![Vertex::default(); 200000]));

    let mut model = TreeModel::Root {
        state: model::ModelState::<Vertex, DynamicAllocHandle<Vertex>>::Dormant(
            SimpleGeometry::init(vec![Vertex::default(); 100000]),
        ),
        sub_handles: Vec::new(),
        ctx: TestContext { point: Vec3::ZERO },
    };

    // model.rotate(glam::Quat::IDENTITY);

    let mut allocater = alloc::BufferDynamicAllocator::<Vertex>::default();

    println!("{:?}", allocater.size());

    let handle = alloc::BufferDynamicAlloc::allocate(&mut allocater, "test", 100000);

    model.wake(handle);

    println!("{:?}", allocater.size());

    //println!("{:?}", gpu_buffer.borrow());

    let now = std::time::Instant::now();

    model.translate(Vec3::new(1.0, 2.0, 3.0));

    let gpu_buffer_clone = gpu_buffer.clone();

    //println!("{:?}", gpu_buffer.borrow());

    allocater.update(|mut mod_action| {
        mod_action.act(
            &mut gpu_buffer_clone.borrow_mut()
                [mod_action.offset..mod_action.offset + mod_action.size],
        );
    });

    println!("{:?}", now.elapsed());

    // println!("{:?}", gpu_buffer.borrow());

    model.destroy();

    println!("{:?}", allocater.size());

    for id in allocater.get_destroyed_handles() {
        allocater.free(&id);
    }

    println!("{:?}", allocater.size());

    panic!("asdas")
}
