use glam::{DQuat, DVec3, U8Vec3};
// use glam::vec2;
// use ndarray;
// let a: ndarray::Array2<i32> = ndarray::Array2::from_shape_vec((2, 3), b).unwrap();
// let b = vec![1, 2, 3, 4, 5, 6];
// let v = vec2(1.0, 1.0);
// println!("{}, {}", v.x, a[(1, 2)]);
use soft_rend::{
    app::SoftwareRenderer,
    pipeline::shader::{Shader, ShaderType},
    resources::loaders::{DefaultMesh, DefaultTexture, MeshLoader, TextureLoader},
    scene::{
        camera::{Camera, CameraStyle},
        light::{Light, LightType},
        Scene,
    },
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
        CameraStyle::FPSLike,
    );
    let mut scene = Scene::with_camera(camera);

    // Create a texture loader/mesh loaders to simplify the creation of textures and meshes.
    let tex_loader = TextureLoader::new();
    let mesh_loader = MeshLoader::new();

    // Populate the scene.
    let mut cube1 = mesh_loader.load_default_mesh(
        DefaultMesh::Cube {
            size: 100.0,
            u_repeat: 5.0,
            v_repeat: 5.0,
        },
        None,
    );
    cube1.translate(DVec3::new(0.0, 0.0, -700.0));
    let checkered_id = scene
        .texture_catalog_mut()
        .add_texture(
            String::from("Checkered"),
            tex_loader.load_default_texture(DefaultTexture::Checkered {
                width: 2,
                height: 2,
                nb_squares_width: 2,
            }),
        )
        .unwrap_or_else(|e| {
            print!("The texture could not be added to the scene: {}", e);
            0
        });
    cube1.set_texture(Some(checkered_id));
    // Create a wall of cubes.
    let side = 7;
    let moves = 110.0;
    for y in -side / 2..=side / 2 {
        for x in -side / 2..=side / 2 {
            let mut c = cube1.clone();
            c.translate(DVec3::new(moves * x as f64, moves * y as f64, 0.0));
            scene.add_mesh(c);
        }
    }

    // Add lights.
    let light = Light::new(
        1.5,
        U8Vec3::new(255, 255, 255),
        LightType::AtInfinity(DVec3::new(0.2, -0.2, -1.0)),
    );
    scene.add_light(light);
    // Decide on the type of shader.
    let shader_type = ShaderType::Flat;
    let shader = match shader_type {
        ShaderType::Phong => todo!("Implement Phong shader"),
        ShaderType::Gouraud => todo!("Implement Gouraud shader"),
        ShaderType::Flat => Shader::new(0.15, ShaderType::Flat),
    };
    // Create and start the app.
    let mut software_renderer = SoftwareRenderer::new(width, height, scene, shader);
    // app.set_max_it(30);
    event_loop.run_app(&mut software_renderer)?;
    Ok(())
}
