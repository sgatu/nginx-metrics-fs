mod counter_fs;
use clap::{Arg, ArgAction, Command};
use fuser::MountOption;

macro_rules! DEFAULT_REGEX_PATTERN {
    () => {
        r#"HTTP\/\d?\.\d?"\s([0-9]{3})"#
    };
}

fn main() {
    let cmd = cli_command();
    let matches = cmd.get_matches();

    let has_pretty = matches.get_flag("pretty");
    let mut mount_path = "/tmp/nginx-stats";
    if let Some(mount_point) = matches.get_one::<String>("mount-path") {
        mount_path = mount_point;
    }
    let mut time_points = 10;
    if let Some(time_pts) = matches.get_one::<String>("time-points") {
        time_points = time_pts.parse::<u16>().ok().unwrap()
    }
    let mut file_name = "stats";
    if let Some(file) = matches.get_one::<String>("file") {
        file_name = file.as_str()
    }
    let mut regex_status_code = DEFAULT_REGEX_PATTERN!().to_owned();
    if let Some(regex) = matches.get_one::<String>("regex") {
        regex_status_code = regex.to_owned()
    }

    let _ = fuser::mount2(
        counter_fs::CounterFS::new(
            file_name.to_owned(),
            has_pretty,
            time_points,
            regex_status_code,
        ),
        mount_path,
        &[MountOption::AutoUnmount, MountOption::AllowOther],
    );
}

fn cli_command() -> Command {
    Command::new("nginx-stats")
        .about("Service used as proxy for nginx logs to get minute by minute status code stats")
        .arg(
            Arg::new("mount-path")
                .short('m')
                .long("mount-path")
                .action(ArgAction::Set)
                .help("Directory where the fuse system will be mounted. Example -m /tmp/stats")
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new("pretty")
                .help("Should create a pretty print file [default: false]")
                .short('p')
                .long("pretty")
                .required(false)
                .num_args(0),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .default_value("stats")
                .help("name of the file to be created with stats")
                .action(ArgAction::Set)
                .required(false)
                .num_args(1),
        )
        .arg(
            Arg::new("time-points")
                .short('t')
                .long("time-points ")
                .action(ArgAction::Set)
                .default_value("10")
                .help("How many minutes of stats should be stored")
                .required(false)
                .num_args(1),
        )
        .arg(
            Arg::new("regex")
                .short('r')
                .long("regex ")
                .action(ArgAction::Set)
                .required(false)
                .help("Regex to match status code in logs")
                .required(false)
                .num_args(1),
        )
}
