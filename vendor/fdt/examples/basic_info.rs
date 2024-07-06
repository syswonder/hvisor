static MY_FDT: &[u8] = include_bytes!("../dtb/test.dtb");

fn main() {
    let fdt = fdt::Fdt::new(MY_FDT).unwrap();

    println!("This is a devicetree representation of a {}", fdt.root().model());
    println!("...which is compatible with at least: {}", fdt.root().compatible().first());
    println!("...and has {} CPU(s)", fdt.cpus().count());
    println!(
        "...and has at least one memory location at: {:#X}\n",
        fdt.memory().regions().next().unwrap().starting_address as usize
    );

    let chosen = fdt.chosen();
    if let Some(bootargs) = chosen.bootargs() {
        println!("The bootargs are: {:?}", bootargs);
    }

    if let Some(stdout) = chosen.stdout() {
        println!("It would write stdout to: {}", stdout.name);
    }

    let soc = fdt.find_node("/soc");
    println!("Does it have a `/soc` node? {}", if soc.is_some() { "yes" } else { "no" });
    if let Some(soc) = soc {
        println!("...and it has the following children:");
        for child in soc.children() {
            println!("    {}", child.name);
        }
    }
}
