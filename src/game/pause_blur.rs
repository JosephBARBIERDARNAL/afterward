use bevy::{
    core_pipeline::{schedule::Core2d, tonemapping::tonemapping, Core2dSystems, FullscreenShader},
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        view::ViewTarget,
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};

const SHADER_PATH: &str = "shaders/pause_blur.wgsl";

pub(super) struct PauseBlurPlugin;

impl Plugin for PauseBlurPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<PauseBlur>::default(),
            UniformComponentPlugin::<PauseBlur>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_pipeline)
            .add_systems(
                Render,
                prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
            )
            .add_systems(
                Core2d,
                post_process.after(tonemapping).in_set(Core2dSystems::PostProcess),
            );
    }
}

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub(super) struct PauseBlur {
    pub(super) radius_pixels: f32,
}

#[derive(Component)]
struct PauseBlurBindGroups {
    a: (TextureViewId, BindGroup),
    b: (TextureViewId, BindGroup),
}

fn prepare_bind_groups(
    mut commands: Commands,
    mut views: Query<(Entity, &ViewTarget, Option<&mut PauseBlurBindGroups>)>,
    pipeline: Option<Res<PauseBlurPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    uniforms: Res<ComponentUniforms<PauseBlur>>,
    render_device: Res<RenderDevice>,
) {
    let Some(pipeline) = pipeline else {
        return;
    };
    let Some(uniform_binding) = uniforms.uniforms().binding() else {
        return;
    };

    let create_bind_group = |texture: &TextureView| {
        (
            texture.id(),
            render_device.create_bind_group(
                "pause_blur_bind_group",
                &pipeline_cache.get_bind_group_layout(&pipeline.layout),
                &BindGroupEntries::sequential((
                    texture,
                    &pipeline.sampler,
                    uniform_binding.clone(),
                )),
            ),
        )
    };

    for (entity, target, mut bind_groups) in &mut views {
        let main = target.main_texture_view();
        let other = target.main_texture_other_view();
        if let Some(bind_groups) = &mut bind_groups {
            if bind_groups.a.0 != main.id() {
                bind_groups.a = create_bind_group(main);
            }
            if bind_groups.b.0 != other.id() {
                bind_groups.b = create_bind_group(other);
            }
        } else {
            commands.entity(entity).insert(PauseBlurBindGroups {
                a: create_bind_group(main),
                b: create_bind_group(other),
            });
        }
    }
}

fn post_process(
    view: ViewQuery<(
        &ViewTarget,
        &DynamicUniformIndex<PauseBlur>,
        &PauseBlurBindGroups,
        &PauseBlur,
    )>,
    pipeline: Option<Res<PauseBlurPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    mut render_context: RenderContext,
) {
    let Some(pipeline) = pipeline else {
        return;
    };
    let (target, uniform_index, bind_groups, settings) = view.into_inner();
    if settings.radius_pixels <= 0.0 {
        return;
    }
    let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
        return;
    };

    let post_process = target.post_process_write();
    let (_, bind_group) = if bind_groups.a.0 == post_process.source.id() {
        &bind_groups.a
    } else {
        &bind_groups.b
    };
    let mut pass = render_context
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("pause_blur_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

    pass.set_pipeline(render_pipeline);
    pass.set_bind_group(0, bind_group, &[uniform_index.index()]);
    pass.draw(0..3, 0..1);
}

#[derive(Resource)]
struct PauseBlurPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "pause_blur_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<PauseBlur>(true),
            ),
        ),
    );
    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("pause_blur_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: fullscreen_shader.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: asset_server.load(SHADER_PATH),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });
    commands.insert_resource(PauseBlurPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}
