#[derive(Parser)]
#[grammar = "parsers/stdout.pest"] // relative to src
pub struct StdoutParser;

#[cfg(test)]
mod test {
    use pest::Parser;

    use parsers::*;
    use zpool::{Health, TopologyBuilder, Zpool};
    use zpool::vdev::{Disk, Vdev};

    #[test]
    fn test_action_single_line() {
        let one_line = " action: The pool can be imported using its name or numeric identifier.\n";
        parses_to! {
            parser: StdoutParser,
            input: one_line,
            rule: Rule::action,
            tokens: [
                action(0, 72, [
                       multi_line_text(9,72)
                ])
            ]
        }
    }

    #[test]
    fn test_action_multiline() {
        let two_lines = " action: The pool cannot be imported. Attach the missing\n        \
                         devices and try again.\n";
        parses_to! {
            parser: StdoutParser,
            input: two_lines,
            rule: Rule::action,
            tokens: [
                action(0, 88, [
                       multi_line_text(9,88)
                ])
            ]
        }
    }

    #[test]
    fn test_naked_good() {
        let stdout_valid_two_disks = r#"pool: naked_test
     id: 3364973538352047455
  state: ONLINE
 action: The pool can be imported using its name or numeric identifier.
 config:

        naked_test             ONLINE
          /vdevs/import/vdev0  ONLINE
          /vdevs/import/vdev1  ONLINE
          "#;

        // let pairs = StdoutParser::parse(Rule::zpool_import,
        // stdout_valid_two_disks).unwrap_or_else(|e| panic!("{}", e));
        // println!("{:#?}", pairs);
        parses_to! {
            parser: StdoutParser,
            input: stdout_valid_two_disks,
            rule: Rule::zpool,
            tokens: [
                zpool(0, 258, [
                    pool_name(0, 17, [name(6,16)]),
                    pool_id(17, 46, [digits(26,45)]),
                    state(46, 62, [state_enum(55, 61)]),
                    action(62, 134, [multi_line_text(71, 134)]),
                    config(134, 143),
                    pool_line(144, 182, [name(152, 162), state_enum(175, 181)]),
                    vdevs(182, 258, [
                        naked_vdev(182, 220, [
                            disk_line(182, 220, [
                                path(192, 211),
                                state_enum(213, 219)
                            ])
                        ]),
                        naked_vdev(220, 258, [
                            disk_line(220, 258, [
                                path(230, 249),
                                state_enum(251, 257)
                            ])
                        ])
                    ])
                ])
            ]
        }
        let mut pairs = StdoutParser::parse(Rule::zpool, stdout_valid_two_disks)
            .unwrap_or_else(|e| panic!("{}", e));
        let pair = pairs.next().unwrap();
        Zpool::from_pest_pair(pair);
    }

    #[test]
    fn test_naked_bad() {
        let stdout_invalid_two_disks = r#"pool: naked_test
     id: 3364973538352047455
  state: UNAVAIL
 status: One or more devices are missing from the system.
 action: The pool cannot be imported. Attach the missing
        devices and try again.
   see: http://illumos.org/msg/ZFS-8000-6X
 config:

        naked_test             UNAVAIL  missing device
          /vdevs/import/vdev0  ONLINE

        Additional devices are known to be part of this pool, though their
        exact configuration cannot be determined.
        "#;
        // let pairs = StdoutParser::parse(Rule::zpool_import,
        // stdout_invalid_two_disks).unwrap_or_else(|e| panic!("{}", e));
        // println!("{:#?}", pairs);
        parses_to! {
            parser: StdoutParser,
            input: stdout_invalid_two_disks,
            rule: Rule::zpool,
            tokens: [
                zpool(0, 356, [
                    pool_name(0, 17, [name(6,16)]),
                    pool_id(17, 46, [digits(26,45)]),
                    state(46, 63, [state_enum(55, 62)]),
                    status(63, 121, [multi_line_text(71, 121)]),
                    action(121, 209, [multi_line_text(130, 209)]),
                    see(209, 252, [url(217, 251)]),
                    config(252, 261),
                    pool_line(262, 317, [name(270, 280), state_enum(293, 300)]),
                    vdevs(317, 356, [
                        naked_vdev(317, 355, [
                            disk_line(317, 355, [
                                path(327, 346),
                                state_enum(348, 354)
                            ])
                        ])
                    ])
                ])
            ]
        }

        let mut pairs = StdoutParser::parse(Rule::zpool, stdout_invalid_two_disks)
            .unwrap_or_else(|e| panic!("{}", e));
        let pair = pairs.next().unwrap();
        Zpool::from_pest_pair(pair);
    }

    #[test]
    fn test_multiple_import() {
        let stdout = r#"pool: naked_test
     id: 3364973538352047455
  state: ONLINE
 action: The pool can be imported using its name or numeric identifier.
 config:

        naked_test             ONLINE
          /vdevs/import/vdev0  ONLINE
          /vdevs/import/vdev1  ONLINE

     pool: naked_test2
     id: 3364973538352047455
  state: ONLINE
 action: The pool can be imported using its name or numeric identifier.
 config:

        naked_test             ONLINE
          /vdevs/import/vdev0  ONLINE
          /vdevs/import/vdev1  ONLINE
          "#;

        let pairs = StdoutParser::parse(Rule::zpools, stdout).unwrap_or_else(|e| panic!("{}", e));
        let mut zpools = pairs.map(|pair| Zpool::from_pest_pair(pair));

        let first = zpools.next().unwrap();
        assert_eq!(first.name(), &String::from("naked_test"));

        let second = zpools.next().unwrap();
        assert_eq!(second.name(), &String::from("naked_test2"));

        let none = zpools.next();
        assert!(none.is_none());
    }

    #[test]
    fn test_status_scrub() {
        let stdout = r#"  pool: bootpool
 state: ONLINE
status: Some supported features are not enabled on the pool. The pool can
        still be used, but some features are unavailable.
action: Enable all features using 'zpool upgrade'. Once this is done,
        the pool may no longer be accessible by software that does not support
        the features. See zpool-features(7) for details.
  scan: scrub repaired 0 in 0 days 00:00:00 with 0 errors on Tue Nov 28 02:04:11 2017
config:

        NAME        STATE     READ WRITE CKSUM
        bootpool    ONLINE       0     0     0
          nvd0p2    ONLINE       0     0     0

errors: No known data errors

  pool: z
 state: ONLINE
status: Some supported features are not enabled on the pool. The pool can
        still be used, but some features are unavailable.
action: Enable all features using 'zpool upgrade'. Once this is done,
        the pool may no longer be accessible by software that does not support
        the features. See zpool-features(7) for details.
  scan: scrub repaired 0 in 0 days 00:01:54 with 0 errors on Tue Nov 28 11:32:55 2017
config:

        NAME          STATE     READ WRITE CKSUM
        z             ONLINE       0     0     0
          nvd0p4.eli  ONLINE       0     0     0

errors: Pretend this is actual error
"#;

        let pairs = StdoutParser::parse(Rule::zpools, stdout).unwrap_or_else(|e| panic!("{}", e));

        let mut zpools = pairs.map(|pair| Zpool::from_pest_pair(pair));
        let first = zpools.next().unwrap();
        assert_eq!(first.name(), &String::from("bootpool"));
        assert!(first.errors().is_none());
        let vdev = &first.topology().vdevs()[0];
        let vdev_expected = Vdev::Naked(Disk::Disk(std::path::PathBuf::from("nvd0p2")));
        assert_eq!(&vdev_expected, vdev);

        let second = zpools.next().unwrap();
        assert_eq!(second.name(), &String::from("z"));

        assert!(second.errors().is_some());

        let none = zpools.next();
        assert!(none.is_none());
    }

    #[test]
    fn test_no_status_line_in_status() {
        let stdout = r#"  pool: tests-12167169401705616934
 state: ONLINE
  scan: none requested
config:

        NAME                   STATE     READ WRITE CKSUM
        tests-12167169401705616934  ONLINE       0     0     0
          /vdevs/import/vdev0  ONLINE       0     0     0

errors: No known data errors
"#;

        let pairs = StdoutParser::parse(Rule::zpools, stdout).unwrap_or_else(|e| panic!("{}", e));
        let mut zpools = pairs.map(|pair| Zpool::from_pest_pair(pair));

        let first = zpools.next().unwrap();
        assert_eq!(first.name(), &String::from("tests-12167169401705616934"));

        let vdev = &first.topology().vdevs()[0];
        let vdev_expected = Vdev::Naked(Disk::File(std::path::PathBuf::from("/vdevs/import/vdev0")));
        assert_eq!(&vdev_expected, vdev);
    }

    #[test]
    fn test_raided_vdev_status() {
        let stdout = r#"  pool: eden
 state: ONLINE
  scan: scrub repaired 0 in 0 days 07:28:14 with 0 errors on Thu Dec 20 07:28:14 2018
config:

        NAME                                            STATE     READ WRITE CKSUM
        eden                                            ONLINE       0     0     0
          mirror-0                                      ONLINE       0     0     0
            gptid/d27f8063-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d2fd0492-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
          raidz1-1                                      ONLINE       0     0     0
            gptid/d27f8063-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d2fd0492-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d37ba27f-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d3fccc31-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d47c7a14-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
          raidz2-2                                      ONLINE       0     0     0
            gptid/d27f8063-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d2fd0492-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d37ba27f-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d3fccc31-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d47c7a14-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
          raidz3-3                                      ONLINE       0     0     0
            gptid/d27f8063-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d2fd0492-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d37ba27f-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d3fccc31-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0
            gptid/d47c7a14-d17d-11e4-9eed-10c37b9d936f  ONLINE       0     0     0

errors: No known data errors
"#;
        let mut pairs = StdoutParser::parse(Rule::zpool, stdout)
            .unwrap_or_else(|e| panic!("{}", e));
        let pair = pairs.next().unwrap();
        let zpool = Zpool::from_pest_pair(pair);

        let mirror_drives = vec![
            Disk::Disk("gptid/d27f8063-d17d-11e4-9eed-10c37b9d936f".into()),
            Disk::Disk("gptid/d2fd0492-d17d-11e4-9eed-10c37b9d936f".into()),
        ];
        let drives = vec![
            Disk::Disk("gptid/d27f8063-d17d-11e4-9eed-10c37b9d936f".into()),
            Disk::Disk("gptid/d2fd0492-d17d-11e4-9eed-10c37b9d936f".into()),
            Disk::Disk("gptid/d37ba27f-d17d-11e4-9eed-10c37b9d936f".into()),
            Disk::Disk("gptid/d3fccc31-d17d-11e4-9eed-10c37b9d936f".into()),
            Disk::Disk("gptid/d47c7a14-d17d-11e4-9eed-10c37b9d936f".into()),
        ];
        let topo = TopologyBuilder::default()
            .vdevs(vec![
                Vdev::Mirror(mirror_drives.clone()),
                Vdev::RaidZ(drives.clone()),
                Vdev::RaidZ2(drives.clone()),
                Vdev::RaidZ3(drives.clone())
            ])
            .build()
            .unwrap();
        //dbg!(zpool.topology());

        assert_eq!(&topo, zpool.topology());
    }

    #[test]
    fn test_degraded_mirror() {
        let stdout = r#"  pool: test
 state: DEGRADED
status: One or more devices has been taken offline by the administrator.
        Sufficient replicas exist for the pool to continue functioning in a
        degraded state.
action: Online the device using 'zpool online' or replace the device with
        'zpool replace'.
  scan: none requested
config:

        NAME                      STATE     READ WRITE CKSUM
        test                      DEGRADED     0     0     0
          mirror-0                DEGRADED     0     0     0
            14808325297596192025  OFFLINE      0     0     0  was /vdevs/vdev0
            /vdevs/vdev1          ONLINE       0     0     0

errors: No known data errors
"#;
        let mut pairs = StdoutParser::parse(Rule::zpool, stdout)
            .unwrap_or_else(|e| panic!("{}", e));
        let pair = pairs.next().unwrap();
        let zpool = Zpool::from_pest_pair(pair);
        assert_eq!(&Health::Degraded, zpool.health());
    }

    #[test]
    fn test_zpools_on_single_zpool() {
        let stdout = r#"  pool: test
 state: DEGRADED
status: One or more devices has been taken offline by the administrator.
        Sufficient replicas exist for the pool to continue functioning in a
        degraded state.
action: Online the device using 'zpool online' or replace the device with
        'zpool replace'.
  scan: none requested
config:

        NAME                      STATE     READ WRITE CKSUM
        test                      DEGRADED     0     0     0
          mirror-0                DEGRADED     0     0     0
            14808325297596192025  OFFLINE      0     0     0  was /vdevs/vdev0
            /vdevs/vdev1          ONLINE       0     0     0

errors: No known data errors
"#;
        let mut pairs = StdoutParser::parse(Rule::zpools, stdout)
            .unwrap_or_else(|e| panic!("{}", e));
        let pair = pairs.next().unwrap();
        let zpool = Zpool::from_pest_pair(pair);
        assert_eq!(&Health::Degraded, zpool.health());
    }

    #[test]
    fn test_tabs_instead_of_8_spaces() {
        let stdout = "  pool: tests-5810578167377116542\n state: DEGRADED\nstatus: One or more devices has been taken offline by the administrator.\n\tSufficient replicas exist for the pool to continue functioning in a\n\tdegraded state.\naction: Online the device using \'zpool online\' or replace the device with\n\t\'zpool replace\'.\n  scan: none requested\nconfig:\n\n\tNAME                      STATE     READ WRITE CKSUM\n\ttests-5810578167377116542  DEGRADED     0     0     0\n\t  mirror-0                DEGRADED     0     0     0\n\t    15825580777360392022  OFFLINE      0     0     0  was /vdevs/vdev3\n\t    /vdevs/vdev4          ONLINE       0     0     0\n\nerrors: No known data errors\n";
        let mut pairs = StdoutParser::parse(Rule::zpool, stdout)
            .unwrap_or_else(|e| panic!("{}", e));
        let pair = pairs.next().unwrap();
        let zpool = Zpool::from_pest_pair(pair);
        assert_eq!(&Health::Degraded, zpool.health());
    }
}