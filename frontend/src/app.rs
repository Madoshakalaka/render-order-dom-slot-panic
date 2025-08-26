use bounce::{helmet::HelmetBridge, BounceRoot};
use glam::Vec2;
use petgraph::Graph;
use stylist::{manager::StyleManager, yew::ManagerProvider};
use yew::{html::PhantomComponent, prelude::*, Properties};
#[cfg(not(target_arch = "wasm32"))]
use {crate::DeviceInfo, crate::SupportedLanguage, bounce::helmet::StaticWriter};

#[derive(Default, Clone, Debug)]
pub struct Rects {
    pub g: Graph<FooNode, ()>,
}

impl PartialEq for Rects {
    fn eq(&self, other: &Self) -> bool {
        let a_ns = self.g.raw_nodes().iter().map(|n| &n.weight);
        let b_ns = other.g.raw_nodes().iter().map(|n| &n.weight);
        let a_es = self
            .g
            .raw_edges()
            .iter()
            .map(|e| (e.source(), e.target(), &e.weight));
        let b_es = other
            .g
            .raw_edges()
            .iter()
            .map(|e| (e.source(), e.target(), &e.weight));
        a_ns.eq(b_ns) && a_es.eq(b_es)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct FooNode {
    pub label: String,
    pub x: f32,
    pub y: f32,
}

pub const NODE_RADIUS: f32 = 4.;

#[derive(Properties, PartialEq)]
pub struct RectProps {
    pub source_node_coordinates: Vec2,
    pub target_node_coordinates: Vec2,
}

#[function_component]
pub fn Rect(props: &RectProps) -> Html {
    let source_node_coordinates = &props.source_node_coordinates;
    let target_node_coordinates = &props.target_node_coordinates;
    let diff = *target_node_coordinates - *source_node_coordinates;
    let angle = -diff.angle_to(Vec2::X).to_degrees();
    let length = diff.length();

    let width = 3.0; // rectangle width

    let my_x = source_node_coordinates.x.to_string();
    let my_y = source_node_coordinates.y.to_string();

    let transform = format!("translate({} {}) rotate({})", my_x, my_y, angle);

    // somehow removing the handler fixes the dom slot bug
    let oncontextmenu = Callback::from(|e: MouseEvent| {
        e.prevent_default();
    });

    html! {
        <rect
            x=0
            y={( -width/2.0 ).to_string()}
            width={length.to_string()}
            height={width.to_string()}
            transform={transform}
            {oncontextmenu}
        />
    }
}

pub const STROKE_COEFF: f32 = 0.25;

pub(crate) fn arrow_endpoints(origin_0: Vec2, origin_1: Vec2, node_radius: f32) -> (Vec2, Vec2) {
    let dir = (origin_1 - origin_0).normalize();
    let offset = dir * node_radius * (1.0 + STROKE_COEFF / 2.0);
    let s = origin_0 + offset;
    let t = origin_1 - offset;
    (s, t)
}

#[derive(Properties, PartialEq)]
pub struct HomePageProps {
    pub graph: UseStateHandle<Rects>,
}

#[function_component]
pub fn HomePage(props: &HomePageProps) -> Html {
    let graph = &props.graph;

    let edges = graph.g.edge_indices().map(|edge_idx| {
        let (source, target) = graph.g.edge_endpoints(edge_idx).unwrap();
        tracing::info!("Rendering edge from {:?} to {:?}", source, target);

        let s = &graph.g[source];
        let t = &graph.g[target];

        let s_coords = Vec2::new(s.x, s.y);
        let t_coords = Vec2::new(t.x, t.y);

        let source_label = s.label.clone();
        let target_label = t.label.clone();
        let key = format!("{}-{}", source_label, target_label);

        let (source_node_coordinates, target_node_coordinates) = arrow_endpoints(s_coords, t_coords, NODE_RADIUS);

        html! {
            <Rect
                key={key}
                source_node_coordinates={source_node_coordinates}
                target_node_coordinates={target_node_coordinates}
            />
        }
    });

    // peculiar: removing the onpointerdown handler here seems to fix the dom slot bug
    html! { <svg xmlns="http://www.w3.org/2000/svg" onpointerdown={|_|{}}>{ for edges }</svg> }
}

pub fn create_triangle_graph() -> petgraph::Graph<FooNode, ()> {
    // three edges seem necessary to trigger the dom slot bug
    let mut graph = petgraph::Graph::<FooNode, ()>::default();
    graph.add_node(FooNode {
        label: "Reason".to_string(),
        x: 50.0,
        y: 15.0,
    });
    graph.add_node(FooNode {
        label: "Interact".to_string(),
        x: 15.0,
        y: 75.0,
    });
    graph.add_node(FooNode {
        label: "Explore".to_string(),
        x: 85.0,
        y: 75.0,
    });
    // form a loop
    graph.add_edge(
        petgraph::graph::NodeIndex::new(0),
        petgraph::graph::NodeIndex::new(1),
        (),
    );
    graph.add_edge(
        petgraph::graph::NodeIndex::new(1),
        petgraph::graph::NodeIndex::new(2),
        (),
    );
    graph.add_edge(
        petgraph::graph::NodeIndex::new(2),
        petgraph::graph::NodeIndex::new(0),
        (),
    );
    graph
}

#[function_component]
pub fn NavAndMain() -> Html {
    let graph = use_state(|| Rects {
        g: create_triangle_graph(),
    });

    html! { <HomePage graph={graph} /> }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Properties, PartialEq)]
pub struct ServerAppProps {
    pub h_writer: StaticWriter,
    pub manager: StyleManager,
    pub device_info: Option<DeviceInfo>,
    pub presenting_language: SupportedLanguage,
}

#[cfg(not(target_arch = "wasm32"))]
#[function_component]
pub fn ServerApp(props: &ServerAppProps) -> Html {
    html! {
        <BounceRoot>
            <ContextProvider<SupportedLanguage> context={props.presenting_language}>
                <ContextProvider<Option<DeviceInfo>> context={props.device_info}>
                    <ManagerProvider manager={props.manager.clone()}>
                        <NavAndMain />
                    </ManagerProvider>
                </ContextProvider<Option<DeviceInfo>>>
            </ContextProvider<SupportedLanguage>>
            <HelmetBridge default_title="Cognet" writer={props.h_writer.clone()} />
        </BounceRoot>
    }
}

#[function_component]
pub fn App() -> Html {
    let style_mgr = (*use_memo((), |_| {
        StyleManager::new().expect("failed to create style manager.")
    }))
    .to_owned();

    html! {
        <BounceRoot>
            <PhantomComponent<FakeSupportedLanguage>>
                <PhantomComponent<FakeDeviceInfo>>
                    <ManagerProvider manager={style_mgr}>
                        <NavAndMain />
                    </ManagerProvider>
                </PhantomComponent<FakeDeviceInfo>>
            </PhantomComponent<FakeSupportedLanguage>>
            <HelmetBridge default_title="Cognet" />
        </BounceRoot>
    }
}

#[function_component]
fn FakeDeviceInfo() -> Html {
    html! {}
}

#[function_component]
fn FakeSupportedLanguage() -> Html {
    html! {}
}
