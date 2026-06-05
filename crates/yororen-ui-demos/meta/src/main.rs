//! `yororen-ui-demos` CLI index. Prints the list of available
//! demo crates with their `cargo run -p <name>` invocation.
//!
//! The individual demo crates remain the primary build path
//! (each is its own `cargo run` target). This binary is a
//! convenience for newcomers who don't yet know the crate
//! names.

fn main() {
    println!("yororen-ui demos:");
    println!();
    for demo in demos() {
        println!("  cargo run -p {:<28}  # {}", demo.crate_name, demo.label);
    }
    println!();
    println!("13 demos total. Workspace member list lives in /Cargo.toml.");
}

struct Demo {
    crate_name: &'static str,
    label: &'static str,
}

const fn demos() -> &'static [Demo] {
    &[
        Demo {
            crate_name: "counter-demo",
            label: "minimal counter",
        },
        Demo {
            crate_name: "todo-demo",
            label: "todo list with state",
        },
        Demo {
            crate_name: "file-browser-demo",
            label: "file browser",
        },
        Demo {
            crate_name: "popover-placement-demo",
            label: "popover placement",
        },
        Demo {
            crate_name: "toast-demo",
            label: "toast notifications",
        },
        Demo {
            crate_name: "tree-expanded-demo",
            label: "tree with expansion",
        },
        Demo {
            crate_name: "i18n-showcase-demo",
            label: "i18n catalog",
        },
        Demo {
            crate_name: "theme-compare-demo",
            label: "theme package switch",
        },
        Demo {
            crate_name: "variant-showcase-demo",
            label: "custom button variants",
        },
        Demo {
            crate_name: "theme-showcase-demo",
            label: "4-way theme render",
        },
        Demo {
            crate_name: "modal-a11y-demo",
            label: "modal accessibility",
        },
        Demo {
            crate_name: "flavor-gallery-demo",
            label: "catppuccin flavor gallery",
        },
        Demo {
            crate_name: "headless-demo",
            label: "headless hooks",
        },
    ]
}
