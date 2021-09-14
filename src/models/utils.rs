fn console_runner(mut app: App) {
    println!("Type stuff into the console");
    for line in io::stdin().lock().lines() {
        {
            let mut input = app.world.get_resource_mut::<Input>().unwrap();
            input.0 = line.unwrap();
        }
        app.update();
    }
}