use avian2d::prelude::*;
use bevy::prelude::*;
use rand;

#[derive(Component)]
pub struct Fruit;

#[derive(Component, Debug, Clone, Eq, PartialEq)]
pub enum FruitType {
    // TODO: もうちょい多かった気がする
    Grape,
    Cherry,
    Mikan,
    Apple,
    Pear,
    Melon,
    Watermelon,
}

impl FruitType {
    pub fn radius(&self) -> f32 {
        match self {
            FruitType::Grape => 20.0,
            FruitType::Cherry => 30.0,
            FruitType::Mikan => 35.0,
            FruitType::Apple => 40.0,
            FruitType::Pear => 50.0,
            FruitType::Melon => 60.0,
            FruitType::Watermelon => 80.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            FruitType::Grape => Color::hsl(270.0, 0.7, 0.6),
            FruitType::Cherry => Color::hsl(0.0, 1.0, 0.5),
            FruitType::Mikan => Color::hsl(30.0, 1.0, 0.5),
            FruitType::Apple => Color::hsl(2.0, 0.65, 0.5),
            FruitType::Pear => Color::hsl(60.0, 0.67, 0.70),
            FruitType::Melon => Color::hsl(120.0, 1.0, 0.75),
            FruitType::Watermelon => Color::hsl(150.0, 0.80, 0.30),
        }
    }

    pub fn choice() -> Self {
        let variants = [FruitType::Grape, FruitType::Cherry, FruitType::Mikan];
        let choice = rand::seq::index::sample(&mut rand::rng(), variants.len(), 1).index(0);
        variants[choice].clone()
    }

    pub fn next_fruit(&self) -> Self {
        match *self {
            FruitType::Grape => FruitType::Cherry,
            FruitType::Cherry => FruitType::Mikan,
            FruitType::Mikan => FruitType::Apple,
            FruitType::Apple => FruitType::Pear,
            FruitType::Pear => FruitType::Melon,
            FruitType::Melon => FruitType::Watermelon,
            FruitType::Watermelon => FruitType::Watermelon, // 例外
        }
    }
}

pub fn spawn_fruit(
    fruit_type: FruitType,
    position: Vec2,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let radius = fruit_type.radius();
    let color = fruit_type.color();

    commands.spawn((
        Fruit,
        fruit_type,
        // Physics
        RigidBody::Dynamic,
        LinearVelocity(Vec2::new(0.0, 0.0)),
        Collider::circle(radius),
        Restitution::new(0.4), // 反発係数
        ColliderDensity(5.0),  // 密度
        CenterOfMass::new(0.0, -0.5),
        CollisionEventsEnabled,
        // Visual
        Mesh2d(meshes.add(Circle::new(radius))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(position.x, position.y, 0.),
    ));
}

/// マージ対象としてマークされたフルーツ
#[derive(Component)]
pub struct WillMergeFruit;

pub fn observe_fruit_collisions(
    _trigger: On<CollisionStart>,
    collisions: Collisions,
    query: Query<Entity, With<Fruit>>,
    fruit_types: Query<&FruitType>,
    mut commands: Commands,
) {
    for fruit in query.iter() {
        for pair in collisions.collisions_with(fruit) {
            let fruit_1 = pair.body1.unwrap();
            let fruit_2 = pair.body2.unwrap();

            if let (Ok(fruit_type_1), Ok(fruit_type_2)) =
                (fruit_types.get(fruit_1), fruit_types.get(fruit_2))
            {
                // 同じフルーツじゃなかったら放置
                if fruit_type_1 != fruit_type_2 {
                    continue;
                }

                // マージ対象としてマーク
                // 次のupdateで処理
                commands.entity(fruit_1).insert(WillMergeFruit);
                commands.entity(fruit_2).insert(WillMergeFruit);
            }
        }
    }
}

pub fn process_fruits_to_merge(
    mut commands: Commands,
    fruits_to_merge: Query<(Entity, &FruitType, &Transform), With<WillMergeFruit>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    collisions: Collisions,
) {
    let mut processed = std::collections::HashSet::new();

    for (fruit_entity, fruit_type, transform) in fruits_to_merge.iter() {
        if processed.contains(&fruit_entity) {
            continue; // すでに処理済み
        }

        for pair in collisions.collisions_with(fruit_entity) {
            let other_entity = if pair.body1.unwrap() == fruit_entity {
                pair.body2.unwrap()
            } else {
                pair.body1.unwrap()
            };

            // 他のフルーツも処理済みならスキップ
            if processed.contains(&other_entity) {
                continue;
            }

            // そのフルーツはマージ対象？
            if !fruits_to_merge.contains(other_entity) {
                continue;
            }

            // 次のフルーツタイプを決定
            let next_fruit_type = fruit_type.next_fruit();
            match fruit_type {
                FruitType::Watermelon => {
                    // スイカだったら削除だけ
                }
                _ => {
                    // TODO: スポーンする場所をいい感じにする
                    // たまにはみ出るので...

                    // 次のフルーツをスポーン
                    spawn_fruit(
                        next_fruit_type,
                        transform.translation.truncate(),
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                    );
                }
            }

            // 元のフルーツを削除
            commands.entity(fruit_entity).despawn();
            commands.entity(other_entity).despawn();

            processed.insert(fruit_entity);
            processed.insert(other_entity);

            // TODO: 得点計算など
        }
    }
}

pub struct FruitsPlugin;

impl Plugin for FruitsPlugin {
    fn build(&self, app: &mut App) {
        fn setup(
            mut commands: Commands,
            mut meshes: ResMut<Assets<Mesh>>,
            mut materials: ResMut<Assets<ColorMaterial>>,
        ) {
            commands.spawn(Camera2d);

            let color = Color::hsl(10., 0.25, 0.7);

            // TODO: あとでいい感じにまとめたい

            // 床を追加する
            commands.spawn((
                // Physics
                RigidBody::Static,
                Collider::rectangle(500., 10.),
                Friction::new(0.2),
                // Visual
                Mesh2d(meshes.add(Rectangle::new(500., 10.))),
                MeshMaterial2d(materials.add(color)),
                Transform::from_xyz(0., -300., 0.),
            ));

            // 壁を追加する
            commands.spawn((
                // Physics
                RigidBody::Static,
                Collider::rectangle(10., 600.),
                Friction::new(0.2),
                // Visual
                Mesh2d(meshes.add(Rectangle::new(10., 600.))),
                MeshMaterial2d(materials.add(color)),
                Transform::from_xyz(-250., 0., 0.),
            ));
            commands.spawn((
                // Physics
                RigidBody::Static,
                Collider::rectangle(10., 600.),
                Friction::new(0.2),
                // Visual
                Mesh2d(meshes.add(Rectangle::new(10., 600.))),
                MeshMaterial2d(materials.add(color)),
                Transform::from_xyz(250., 0., 0.),
            ));

            commands.spawn((
                Text::new("Press space to toggle wireframes"),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(12),
                    left: px(12),
                    ..default()
                },
            ));

            // Observer は一度だけ登録する
            commands.add_observer(observe_fruit_collisions);
        }

        fn update(
            mut commands: Commands,
            mut meshes: ResMut<Assets<Mesh>>,
            mut materials: ResMut<Assets<ColorMaterial>>,
            mouse: Res<ButtonInput<MouseButton>>,
            window: Single<&Window>,
            camera: Query<(&Camera, &GlobalTransform)>,
        ) {
            let (camera, camera_transform) = camera.single().unwrap();

            // ランダムな位置にランダムな大きさ、色の円を追加
            if mouse.just_pressed(MouseButton::Left) {
                if let Some(position) = window
                    .cursor_position()
                    .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
                {
                    let fruit_type = FruitType::choice();

                    spawn_fruit(
                        fruit_type,
                        position,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                    );
                }
            }
        }

        fn on_added(fruits: Query<Entity, (With<Fruit>, Added<Fruit>)>) {
            for fruit in fruits.iter() {
                println!("A new fruit was added!: {:?}", fruit);
            }
        }

        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (update, on_added.after(update), process_fruits_to_merge),
        );

        app.add_plugins(MeshPickingPlugin);

        app.insert_resource(Gravity(Vec2::NEG_Y * 981.0));
    }
}
