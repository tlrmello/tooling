# ip_to_acl

ip_to_acl interfaces with the fastly api in order to ingest a json file containing a list of ip addresses and copy each entry into an ACL object on a given Fastly service.

the tool requires a service ID, and fastly api token both which are added as constants within the main rs file.

the json file containing the IP addresses _must_ follow the below format:

```
{
	"ip_list": [
		"127.0.0.0",
        	"127.0.0.1/24",
        	"1000:0000:0000:0000:0000:0000:0000:0000",
        	"1000:0000:0000:0000:0000:0000:0000:0000/32"
	]
}
```

both ipv4 and ipv6 are supported, as are cidr ranges, and please submit PRs if you're willing to help improve upon my awfully beginner rust code.

suggestions for improvements: 

* batch address upload (https://developer.fastly.com/reference/api/acls/acl-entry/#bulk-update-acl-entries)
* retry upload on network error/failure
* pushing entries to an already created ACL 
* providing an ACL to use opposed to creating a new one
* deleting or overwriting existing ACL
* allowing the user to provide a name for the ACL to be created
* moving away from in-line configuration vars to user-provided arguments
