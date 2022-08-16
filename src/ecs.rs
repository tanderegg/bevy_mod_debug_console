use std::fmt::Write;

use bevy::{
    ecs::{
        archetype::{ArchetypeId, Archetypes},
        component::{ComponentId, Components, StorageType},
        entity::{Entities, Entity},
    },
    utils::get_short_name,
};
use clap::{App, Arg, AppSettings, ArgGroup, ArgMatches};

pub fn list_resources(archetypes: &Archetypes, components: &Components) -> String {
    let mut output = String::new();

    let mut r: Vec<String> = archetypes
        .resource()
        .components()
        .map(|id| components.get_info(id).unwrap())
        // get_short_name removes the path information
        // i.e. `bevy_audio::audio::Audio` -> `Audio`
        // if you want to see the path info replace
        // `get_short_name` with `String::from`
        .map(|info| get_short_name(info.name()))
        .collect();

    // sort list alphebetically
    r.sort();

    writeln!(output, "[resource name]").unwrap();
    r.iter()
        .for_each(|name| writeln!(output, "{}", name).unwrap());

    output
}

fn get_components_by_name(
    components: &Components,
    short: bool,
    filter: Option<&str>,
) -> Vec<(usize, String)> {
    let mut names = Vec::new();
    for id in 1..components.len() {
        if let Some(info) = components.get_info(ComponentId::new(id)) {
            if short {
                names.push((id, get_short_name(info.name())));
            } else {
                names.push((id, String::from(info.name())));
            }
        }
    }

    if let Some(filter) = filter {
        names
            .iter()
            .cloned()
            .filter(|(_, name)| name.contains(filter))
            .collect()
    } else {
        names
    }
}

fn list_components(c: &Components, short: bool, filter: Option<&str>) -> String {
    let mut names = get_components_by_name(c, short, filter);
    names.sort();

    let mut output = String::new();
    writeln!(output, "[component id] [component name]").unwrap();
    names
        .iter()
        .for_each(|(id, name)| writeln!(output, "{} {}", id, name).unwrap());

    output
}

fn list_entities(e: &Entities) -> String {
    let mut output = String::new();
    writeln!(output, "[entity index] [archetype id]").unwrap();
    for id in 0..e.len() {
        if let Some(entity) = e.resolve_from_id(id) {
            if let Some(location) = e.get(entity) {
                writeln!(output, "{} {}", id, location.archetype_id.index()).unwrap();
            }
        }
    }

    output
}

fn list_archetypes(a: &Archetypes) -> String {
    let mut output = String::new();
    writeln!(output, "[id] [entity count]").unwrap();
    a.iter().for_each(|archetype| {
        writeln!(output,
            "{} {}",
            archetype.id().index(),
            archetype.entities().iter().count()
        ).unwrap()
    });

    output
}

fn print_ecs_counts(a: &Archetypes, c: &Components, e: &Entities) -> String {
    format!(
        "entities: {}, components: {}, archetypes: {}\n",
        e.len(),
        c.len(),
        a.len()
    )
}

fn find_archetypes_by_component_name(
    a: &Archetypes,
    c: &Components,
    component_name: &str,
) -> String {
    let components = get_components_by_name(c, false, Some(component_name));

    if components.is_empty() {
        return format!("No component found with name {}\n", component_name);
    }

    if components.len() > 1 {
        let mut output = String::new();
        writeln!(
            output,
            "More than one component found with name {}",
            component_name
        ).unwrap();
        writeln!(
            output,
            "Consider searching with '--componentid' instead\n"
        ).unwrap();
        writeln!(output, "[component id] [component name]").unwrap();
        components
            .iter()
            .for_each(|(id, name)| writeln!(output, "{} {}", id, name).unwrap());
        return output;
    }

    if let Some(id_name) = components.get(0) {
        return find_archetypes_by_component_id(a, id_name.0);
    };

    // should never be hit as clap
    String::from("unsupported command")
}

fn find_archetypes_by_component_id(a: &Archetypes, component_id: usize) -> String {
    let mut output = String::new();

    let archetypes = a
        .iter()
        .filter(|archetype| archetype.components().any(|c| c.index() == component_id))
        .map(|archetype| archetype.id().index());

    writeln!(output, "archetype ids:").unwrap();
    archetypes.for_each(|id| write!(output, "{}, ", id).unwrap());
    output.push('\n');

    output
}

pub fn get_archetype_id_by_entity_id(a: &Archetypes, entity_id: u32) -> Option<usize> {
    let mut archetypes = a
        .iter()
        .filter(|archetype| archetype.entities().iter().any(|e| e.id() == entity_id))
        .map(|archetype| archetype.id().index());

    archetypes.next()
}

fn find_archetype_by_entity_id(a: &Archetypes, entity_id: u32) -> String {
    let mut output = String::new();

    let archetype_id = get_archetype_id_by_entity_id(a, entity_id);

    writeln!(output, "archetype id:").unwrap();
    if let Some(id) = archetype_id {
        writeln!(output, "{}", id).unwrap()
    }

    output
}

fn find_entities_by_component_id(a: &Archetypes, component_id: usize) -> String {
    let entities: Vec<&Entity> = a
        .iter()
        .filter(|archetype| archetype.components().any(|c| c.index() == component_id))
        .flat_map(|archetype| archetype.entities())
        .collect();

    if entities.iter().len() == 0 {
        let mut output = String::new();
        writeln!(output, "no entites found").unwrap();
        return output;
    }

    let mut output = String::new();
    writeln!(output, "entity ids:").unwrap();
    entities
        .iter()
        .for_each(|id| write!(output, "{}, ", id.id()).unwrap());
    output.push('\n');

    output
}

fn find_entities_by_component_name(a: &Archetypes, c: &Components, component_name: &str) -> String {
    let components = get_components_by_name(c, false, Some(component_name));

    let mut output = String::new();
    components.iter().for_each(|(id, name)| {
        writeln!(output, "{}", name).unwrap();
        output.push_str(&find_entities_by_component_id(a, *id));
        output.push('\n');
    });

    output
}

fn print_archetype(a: &Archetypes, c: &Components, archetype_id: ArchetypeId) -> String {
    let mut output = String::new();
    if let Some(archetype) = a.get(archetype_id) {
        writeln!(output, "id: {:?}", archetype.id()).unwrap();
        writeln!(output, "table_id: {:?}", archetype.table_id()).unwrap();
        write!(
            output,
            "entities ({}): ",
            archetype.entities().iter().count()
        ).unwrap();
        archetype
            .entities()
            .iter()
            .for_each(|entity| writeln!(output, "{}, ", entity.id()).unwrap());
        writeln!(output).unwrap();
        // not sure what entity table rows is, so commenting out for now
        // print!(
        //     "entity table rows ({}): ",
        //     archetype.entity_table_rows().iter().count()
        // );
        // archetype
        //     .entity_table_rows()
        //     .iter()
        //     .for_each(|row| print!("{}, ", row));
        // println!("");
        write!(
            output,
            "table_components ({}): ",
            archetype.table_components().iter().count()
        ).unwrap();
        archetype
            .table_components()
            .iter()
            .map(|id| (id.index(), c.get_info(*id).unwrap()))
            .map(|(id, info)| (id, get_short_name(info.name())))
            .for_each(|(id, name)| write!(output, "{} {}, ", id, name).unwrap());
        output.push('\n');

        write!(
            output,
            "sparse set components ({}): ",
            archetype.sparse_set_components().iter().count()
        ).unwrap();
        archetype
            .sparse_set_components()
            .iter()
            .map(|id| (id.index(), c.get_info(*id).unwrap()))
            .map(|(id, info)| (id, get_short_name(info.name())))
            .for_each(|(id, name)| write!(output, "{} {}, ", id, name).unwrap());
        writeln!(output).unwrap();
    } else {
        writeln!(
            output,
            "No archetype found with id: {}",
            archetype_id.index()
        ).unwrap();
    }

    output
}

fn print_component(c: &Components, component_id: usize) -> String {
    let mut output = String::new();
    if let Some(info) = c.get_info(ComponentId::new(component_id)) {
        writeln!(output, "Name: {}", info.name()).unwrap();
        writeln!(output, "Id: {}", info.id().index()).unwrap();
        write!(output, "StorageType: ").unwrap();
        match info.storage_type() {
            StorageType::Table => output.push_str("Table\n"),
            StorageType::SparseSet => output.push_str("SparseSet\n"),
        }
        writeln!(output, "SendAndSync: {}", info.is_send_and_sync()).unwrap();
    } else {
        write!(output, "No component found with id: {}", component_id).unwrap();
    }

    output
}

fn print_component_by_name(c: &Components, component_name: &str) -> String {
    let components = get_components_by_name(c, false, Some(component_name));

    let mut output = String::new();
    components
        .iter()
        .for_each(|(id, _)| writeln!(output, "{}", &print_component(c, *id)).unwrap());

    output
}

pub fn build_commands(app: App) -> App {
    let app = app.subcommand(
            App::new("counts").about("print counts of archetypes, components, and entities"),
        )
        .subcommand(
            App::new("archetypes")
                .about("get archetypes info")
                .alias("archetype")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(App::new("list")
                    .about("list all archetypes")
                )
                .subcommand(App::new("info")
                    .about("get info of one archetype")
                    .arg(Arg::new("id")
                        .long("id")
                        .value_name("Id")
                        .help("id to get"))
                    .group(ArgGroup::new("search params")
                        .args(&["id"])
                        .required(true)
                    )
                )
                .subcommand(App::new("find")
                    .about("find a archetype")
                    .arg(Arg::new("componentid")
                        .long("componentid")
                        .value_name("ComponentId")
                        .help("find types that have components with ComponentId"))
                    .arg(Arg::new("componentname")
                        .long("componentname")
                        .value_name("ComponentName")
                        .help("find types that have components with ComponentName"))
                    .arg(Arg::new("entityid")
                        .long("entityid")
                        .value_name("EntityId")
                        .help("find types that have entities with EntityId"))
                    .group(ArgGroup::new("search params")
                        .args(&["componentid", "componentname", "entityid"])
                        .required(true)
                    )
                )
        )
        .subcommand(
            App::new("components")
                .about("get components info")
                .alias("component")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(App::new("list")
                    .about("list all components")
                    .arg(Arg::new("filter")
                        .short('f')
                        .long("filter")
                        .value_name("Filter")
                        .help("filter list"))
                    .arg(Arg::new("long")
                        .short('l')
                        .long("long")
                        .help("display long name")),
                )
                .subcommand(App::new("info")
                    .about("get info of one component")
                    .arg(Arg::new("id")
                        .long("id")
                        .value_name("Id")
                        .help("id to get"))
                    .arg(Arg::new("name")
                        .long("name")
                        .value_name("Name")
                        .help("name to get"))
                    .group(ArgGroup::new("search params")
                        .args(&["id", "name"])
                        .required(true)
                    )
                )
        )
        .subcommand(
            App::new("entities")
                .about("get entity info")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    App::new("list")
                        .about("list all entities")
                )
                .subcommand(
                    App::new("find")
                        .about("find entity matching search params")
                        .arg(Arg::new("componentid")
                            .long("componentid")
                            .value_name("ComponentId")
                            .help("find types that have components with ComponentId"))
                        .arg(Arg::new("componentname")
                            .long("componentname")
                            .value_name("ComponentName")
                            .help("find types that have components with ComponentName"))
                        .group(ArgGroup::new("search params")
                            .args(&["componentid", "componentname"])
                            .required(true)
                        )
                )
        )
        .subcommand(
            App::new("resources")
                .about("get resource info")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    App::new("list")
                        .about("list all resources")
                )
        );

    app
}

pub fn match_commands(
    matches: &ArgMatches,
    a: &Archetypes,
    c: &Components,
    e: &Entities,
) -> String {
    match matches.subcommand() {
        Some(("archetypes", matches)) => match matches.subcommand() {
            Some(("list", _)) => list_archetypes(a),
            Some(("find", matches)) => {
                if let Ok(component_id) = matches.value_of_t("componentid") {
                    find_archetypes_by_component_id(a, component_id)
                } else if let Some(component_name) = matches.value_of("componentname") {
                    find_archetypes_by_component_name(a, c, component_name)
                } else if let Ok(entity_id) = matches.value_of_t("entityid") {
                    find_archetype_by_entity_id(a, entity_id)
                } else {
                    // should never be hit as clap checks this
                    String::from("this line should not be hittable")
                }
            }
            Some(("info", matches)) => {
                if let Ok(id) = matches.value_of_t("id") {
                    print_archetype(a, c, ArchetypeId::new(id))
                } else {
                    String::from("this line should not be hittable")
                }
            }
            _ => String::from("this line should not be hittable"),
        },
        Some(("components", matches)) => match matches.subcommand() {
            Some(("list", matches)) => {
                list_components(c, !matches.is_present("long"), matches.value_of("filter"))
            }
            Some(("info", matches)) => {
                if let Ok(id) = matches.value_of_t("id") {
                    print_component(c, id)
                } else if let Some(name) = matches.value_of("name") {
                    print_component_by_name(c, name)
                } else {
                    String::from("this line should not be hittable")
                }
            }
            _ => String::from("this line should not be hittable"),
        },
        Some(("entities", matches)) => match matches.subcommand() {
            Some(("list", _)) => list_entities(e),
            Some(("find", matches)) => {
                if let Ok(component_id) = matches.value_of_t("componentid") {
                    find_entities_by_component_id(a, component_id)
                } else if let Some(component_name) = matches.value_of("componentname") {
                    find_entities_by_component_name(a, c, component_name)
                } else {
                    String::from("this line should not be hittable")
                }
            }
            _ => String::from("this line should not be hittable"),
        },
        Some(("resources", matches)) => match matches.subcommand() {
            Some(("list", _)) => list_resources(a, c),
            _ => String::from("this line should not be hittable"),
        },
        Some(("counts", _)) => print_ecs_counts(a, c, e),
        _ => String::from(""),
    }
}
