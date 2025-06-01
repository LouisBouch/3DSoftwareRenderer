use glam::{DQuat, DVec3};
// use glam::vec2;
// use ndarray;
// let a: ndarray::Array2<i32> = ndarray::Array2::from_shape_vec((2, 3), b).unwrap();
// let b = vec![1, 2, 3, 4, 5, 6];
// let v = vec2(1.0, 1.0);
// println!("{}, {}", v.x, a[(1, 2)]);
use soft_rend::{
    app::App,
    resources::loaders::{DefaultMesh, DefaultTexture, MeshLoader, TextureLoader},
    scene::{camera::Camera, Scene},
};
use winit::event_loop::EventLoop;
fn main() -> Result<(), winit::error::EventLoopError> {
    let (width, height) = (800, 600);
    // Create the event loop that will be used to manage window events.
    let event_loop = EventLoop::new().unwrap();

    // Create the scene that will be rendered.
    let camera = Camera::new_perspective(
        &DVec3::ZERO,
        &DQuat::IDENTITY,
        5.0,
        1000.0,
        width as f32 / height as f32,
        90.0,
    );
    let mut scene = Scene::with_camera(camera);

    // Create a texture loader/mesh loaders to simplify the creation of textures and meshes.
    let tex_loader = TextureLoader::new();
    let mesh_loader = MeshLoader::new();

    // Populate the scene.
    let mut cube = mesh_loader.load_default_mesh(
        DefaultMesh::Cube {
            size: 100.0,
            u_repeat: 5.0,
            v_repeat: 5.0,
        },
        None,
    );
    cube.translate(DVec3::new(0.0, 0.0, -70.0));
    let checkered_id = scene
        .texture_catalog_mut()
        .add_texture(
            String::from("Checkered"),
            tex_loader.load_default_texture(DefaultTexture::Checkered {
                width: 20,
                height: 20,
                nb_squares_width: 2,
            }),
        )
        .unwrap_or_else(|e| {
            print!("The texture could not be added to the scene: {}", e);
            0
        });
    cube.set_texture(Some(checkered_id));
    scene.add_mesh(cube);

    // Create and start the app.
    let mut app = App::new(width, height, scene);
    // app.set_max_it(30);
    event_loop.run_app(&mut app)?;
    Ok(())
}
