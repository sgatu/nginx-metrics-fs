### Nginx metrics filesystem
---
#### 0. About

Simple FUSE service in charge of counting NGINX response codes for a realtime, simpler and less resource intensive global stats.

#### 1. Why?

Because I can, and because nginx metrics without parsing the access.log are only available in NGINX Plus. This will give you real time minute by minute metrics for your nginx instance in a nice parseable format. 


#### 2. How it works?

This application creates a virtual filesystem using FUSE with a special file that replaces (or work in parallel) the nginx access.log file and where each new access line written to that file will be parsed and counted by minute. The amount of minutes stores is 10 by default, but can be configured using the **--time-points** parameter.

In your nginx configuration all you have to do is to add a new access_log entry in your **server** block and then restart your nginx service.

```nginx.conf
server {
    ...
    access_log /your/mount/path/stats
    ...
}
```

Then you can view the stats just by printing that file contents.
```
cat /your/mount/path/stats
```
Output
```
|dt|HTTP_100|HTTP_200|HTTP_300|HTTP_400|HTTP_500|
|06-11-2023 17:54:00|0|0|0|0|0|
```

##### 2.a IMPORTANT
Filesystem must be mounted before nginx starts, if not nginx will not retry to open the file if at start does not exist. Not sure if a workaround is possible.

#### 3. Arguments

If you run the application with the parameter -h or --help you will get a info about all the commands, and if optional their details. Like the following:
```
Service used as proxy for nginx logs to get minute by minute status code stats

Usage: nginx-stats [OPTIONS] --mount-path <mount-path>

Options:
  -m, --mount-path <mount-path>     Directory where the fuse system will be mounted. Example -m /tmp/stats
  -p, --pretty                      Should create a pretty print file [default: false]
  -f, --file <file>                 name of the file to be created with stats [default: stats]
  -t, --time-points  <time-points>  How many minutes of stats should be stored [default: 10]
  -r, --regex  <regex>              Regex to match status code in logs
  -h, --help                        Print help
```

The only required parameter is -m or --mount-path which will be the path where the virtual file will be created, it should be a folder path and the path must exist.

If the -p or --pretty flag is used then it will create a second virtual file called pretty_{filename} which will output a more human friendly version of the stats.
```
|       DateTime       |    # HTTP 100    |    # HTTP 200    |    # HTTP 300    |    # HTTP 400    |    # HTTP 500    |
| 06-11-2023 18:05:00  |        0         |        6         |        0         |        2         |        0         |
```

By default the file names are "stats" and "pretty_stats", but this can be changed using the -f or --file argument. For example with **-f nginx_stats** you will get a file called nginx_stats and if the --pretty flag is set then it will create a second file called pretty_nginx_stats.

#### 4. Future ideas or ToDo

- Count responses by more specific codes, like 404, or 401 instead of the actual grouping by 4xx. 
- Maybe add filters, maybe be able to filter by endpoint, but maybe can be done with nginx if= argument too.
- Add some testing


#### 5. Contributing
Just fork and PR. Contributions are welcome, whether they are bug fixes, improvements to the code, or new features.