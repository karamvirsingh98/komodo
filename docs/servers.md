# connecting servers

Integrating a device into the monitor system has 2 steps:

 1. Setup and start the periphery agent on the server
 2. Adding the server to monitor via the core API

## setup monitor periphery

The easiest way to do this is to follow the [monitor guide](https://github.com/mbecker20/monitor-guide). This is a repo containing directions and scripts enabling command line installation via ssh or remotely.

### manual install steps

 1. Download the periphery binary from the latest [release](https://github.com/mbecker20/monitor/releases).

 2. Create and edit your config files, following the [config example](https://github.com/mbecker20/monitor/blob/main/config_example/periphery.config.example.toml). The monitor cli can be used to add the boilerplate: ```monitor periphery gen-config --path /path/to/config.toml```. The files can be anywhere, and can be passed to periphery via the ```--config-path``` flag.

 3. Ensure that inbound connectivity is allowed on the port specified in periphery.config.toml (default 8000).

 4. Install docker. Make sure whatever user periphery is run as has access to the docker group without sudo.

 5. Start the periphery binary with your preferred process manager, like systemd. The config read from the file is printed on startup, ensure that it is as expected.

## example periphery start command

```
periphery \
	--config-path /path/to/periphery.config.base.toml \
	--config-path /other_path/to/periphery.config.overide.toml \
	--merge-nested-config \
	--home_dir /home/username
```

## passing config files

when you pass multiple config files, later --config-path given in the command will always overide previous ones.

there are two ways to merge config files. The default behavior is to completely replace any base fields with whatever fields are present in the overide config. So if you pass ```allowed_ips = []``` in your overide config, the final allowed_ips will be an empty list as well. 

```--merge-nested-config``` will merge config fields recursively and extend config array fields. 

For example, with ```--merge-nested-config``` you can specify an allowed ip in the base config, and another in the overide config, they will both be present in the final config.

Similarly, you can specify a base docker / github account pair, and extend them with additional accounts in the overide config.

## adding the server to monitor

The easiest way to add the server is with the GUI. On the home page, click the + button to the right of the server search bar, configure the name and address of the server. The address is the full http/s url to the periphery server, eg http://12.34.56.78:8000.

Once it is added, you can use access the GUI to modify some config, like the alerting thresholds for cpu, memory and disk usage. A server can also be temporarily disabled, this will prevent alerting if it goes offline.

Since no state is stored on the periphery servers, you can easily redirect all deployments to be hosted on a different server. Just update the address to point to the new server.

[next: building](https://github.com/mbecker20/monitor/blob/main/docs/builds.md)

[back to table of contents](https://github.com/mbecker20/monitor/blob/main/readme.md)