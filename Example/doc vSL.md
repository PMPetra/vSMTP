
<header><h1> vSL : vSMTP scripting language</header>

# Introduction
vSL is a lightweight scripting language dedicated to email filtering. It is based on the RHAI scripting language. vSL combines declarative rules with objects and actions.

vSL has no notion of a "main" program. vSL files are analyzed and executed by vMTA through specific function calls. However, advanced users can use the RHAI scripting language directly in vSL files to create and manage a wide variety of actions.

<br>

vSL is free software and is provided as usual without any warranty, as stated in its license.

<br>

# Objects

Objectfs are declared with the "obj" token. Two syntax are available.  
The inline syntax :
```
obj type "name" "value";
```

```
obj ip4 "my_host" "192.168.1.34";
obj fqdn "local_domain" "foo.bar";
```

The extended syntax, allowing the use of user-defined fields:
```
obj type "name" #{  
	value: "value",                  
	<user_field1>: "value",  
	...  
	<user_fieldn>: "value",  
};
```
```
obj ip4 "local_MDA" #{
	value: "192.168.0.34",            
	color: "bbf3ab",
	description: "Internal delivery agent"
};
```
> Note that the last comma is not mandatory.  

<br>

The following type of objects are supported natively:
| Type | Description | Syntax | Comment 
| :--- | :--- | :--- | :--- 
| val | Untyped value | string | Bind a value
| ip4 | IPv4 address | x.y.z.t 
| rg4 | IPv4 range | x.y.z.t/rg 
| ip6 | IPv6 address | x.y.z.t | Not implemented in RC1
| rg6 | IPv6 range | x.y.z.t/rg | Not implemented in RC1
| addr | email address | user@domain | ???? quid des users locaux ???
| fqdn | Fully qualified domain name
| regex | Regular expression | | PERL regex syntax
| grp | A group of objects | | See group section
| file:\<obj\> | A file of objects | PATH:/file | See File section


## Files
File objects are standard Unix text files containing values delimited by CRLF.  
Only one type of object is authorized and must be declared after the keyword "file:". 
```
obj file:addr "local_MTA" "/var/vmta/config/local_mta.txt";
```
```
# cat /var/vmta/config/local_mta.txt
192.168.1.10
192.168.1.12
10.3.4.240
```

## Groups   // TO DO
Groups are collections of objects.  
```
obj ip4 "my_host" "192.168.1.34";
fqdn "local_domain" "foo.bar";

obj grp "bar" [
	obj addr "new_relay" "192.145.2.3",
	"my_host",
	"local_domain"
]

obj grp "foobar" [
	obj addr "foo" "192.145.2.55",
	"bar",
]
```
> Please note that unlike objects where fields are declared between parentheses, groups use squared brackets.

<br>

# Actions
Interaction with SMTP traffic is done through actions predefined by vSL. 

## Rule engine actions

| Action | Description | Syntax | Comment 
| :--- | :--- | :--- | :--- 
| ACCEPT | Accept | ACCEPT() | Move to the next SMTP state 
| FACCEPT | Force accept | FACCEPT() | Skip all rules, deliver the mail 
| CONTINUE | Continue processing | CONTINUE() | Jump to the next rule
| DENY | Deny processing | DENY() | Send a valid SMTP return code 
| STOP | Stop processing | STOP() | Close the session

## Other actions

| Action | Syntax | Description 
| :--- | :--- | :--- 
| LOG | Accept | LOG(string, PATH:/file) | Log, append string to PATH:/file 
| SLOG | syslog ????
| BCC | BCC(address) | Blind carbon copy
| MAIL | MAIL(from, to, subject, body) | Body must be a TXT string
| WRITE | WRITE(file) | Write a raw copy of the mail 
| DUMP | DUMP(file) | Write a JSON-like copy of the mail
| EXTERN | | Call external prog ??????

```
vsl.LOG(`Hello world !!!`, /tmp/my_file);
```

Actions can be chained :
```
vsl.LOG(`Hello world !!!`, /tmp/my_file);   
vsl.DUMP(`/tmp/mail/dump/`);   
vsl.FACCEPT();
```
> Please note that vSL actions must be prefixed in rules. See the chapter on advanced programming for more explanation.

## User-defined actions

Combined actions can be declared using a RHAI function. 

```
fn my_trigger() {
	vsl.LOG(`Hello world !!!`, /tmp/my_file);
	vsl.DUMP(`/tmp/mail/dump/`);   
	vsl.FACCEPT()
}

fn my_sequence() {
	vsl.LOG(`Hello world !!!`, /tmp/my_file);
	vsl.DUMP(`/tmp/mail/dump/`);   
	vsl.FACCEPT();
	return 42;
}
...

my_action();
```
> Please notice that my_trigger function return FACCEPT (no comma) whereas my_sequence only execute the actions.


<br>

# Rules
>Rules are the entry point to blahblah....
>
>Il faut expliquer que certaines règles sont embedded dans le moteur (#caractères, check du format, timeout, retry etc.)\
>A valider la possibilité de les modifier ici ? ou dans le TOML ?

## SMTP states 
Using rules, vSL can analyze and interact with the SMTP traffic at multiple stages. 

| Stage | SMTP state | Context available
| :--- | :--- | :--- 
| connect | Before HELO/EHLO command | Source/Destination IP address and ports
| helo | After HELO/EHLO command | connect + HELO string ?parameters (quid du starttls ?)
| mail | After MAIL FROM command | helo + sender address 
| rcpt | After RCPT TO command | The SMTP envelop 
| preq | Before queuing the mail.<br>Connection is not closed.<br> Client is waiting for SMTP return code. | The whole mail
| postq | After queuing the mail.<br>Connection closed.<br> SMTP code sent. | The whole mail

<br>

## Syntax
Rules follow a specific syntax :
```
rule <state> <name> #{
	condition: || <condition>,
	on_success: || <action>,
	on_failure: || <action>,
};
```

Rules must return a vSL "rule engine" action.

## Built-in VSL conditions
Foreach stage a VSL condition that match the context is available.
The function syntax is : IS_*STAGE*(object).

```
obj addr "foo" "foo@bar.com";

[...] vsl.IS_CONNECT("192.168.1.34")
[...] vsl.IS_MAIL("foo")
```

| Stage | SMTP state | Context available
| :--- | :--- | :--- 
| connect | Before HELO/EHLO command | Source/Destination IP address and ports
| helo | After HELO/EHLO command | connect + HELO string ?parameters (quid du starttls ?)
| mail | After MAIL FROM command | helo + sender address 
| rcpt | After RCPT TO command | The SMTP envelop 
| preq | Before queuing the mail.<br>Connection is not closed.<br> Client is waiting for SMTP return code. | The whole mail
| postq | After queuing the mail.<br>Connection closed.<br> SMTP code sent. | The whole mail

```
obj ip4 "localhost" "192.168.1.34";

rule connect "check on connect" #{
	condition:  || vsl.IS_CONNECT("localhost"),
	on_success: || vsl.ACCEPT(),
	on_failure: || vsl.DENY()
};
```



The context is available at any stage.

```
obj ip4 "localhost" "192.168.1.34";
obj addr "foo" "foo@bar.com";

rule mail "adv check" #{
	condition:  || vsl.IS_CONNECT("localhost") && vsl.IS_MAIL("foo"),
	on_success: || vsl.ACCEPT(),
	on_failure: || vsl.DENY()
};
```


<br>

### Conditions

The **"condition: ||"** primitive expects a boolean after the || symbol.  
Booleans can come directly from RHAI or vSL functions as shown hereunder.

```
condition: true    
```


```
condition: || vsl.IS_CONNECT("10.0.0.1")
```
```
obj fqdn "foobar" "my.foo.bar";

fn my_function(x) {
	if (x == "foo") { true } else { false }
}

[...]
rule mail "adv check" #{
	condition: || !vsl.IS_HELO("foobar") && my_function("bar")
[...]
```
> Remark : && and || operators are short-circuits.  
> In this case foobar() function will not be evaluated if the 1st part already proves the condition wrong.  
> To counter this behavior use the boolean operators & and |.

<br>

### On_success and on_failure 
These primitives must return a vSL "rule engine" action.
However, actions can be chained using parenthesis or a group.
```
obj ip4 "localhost" "192.168.1.34";

fn my_action() {
	vsl.DUMP(`/tmp/mail/dump/`),   
	vsl.FACCEPT()
}

rule connect "check on connect" #{
	condition:  || vsl.IS_CONNECT("localhost"),
    on_success: || my_action(),
    on_failure: || { 
    	vsl.LOG(`Connection from this host is not allowed.`, "stdout"); 
    	vsl.DENY() 
	},
};
```
> Note the absence of the semicolon after DENY() since the rule must return a "rule engine" action. 

<br>

# Advanced scripting

## Built-in environment variables

The SMTP environment and the email body is available for analysis through built-in variables.

| Stage | Name | Type | Description |
| :--- | :--- | :--- | :--- |
| connect | ${source_ip} | ? | ?
| connect | ${dest_ip} |
| connect | ${source_port} |
| connect | ${dest_port} |
| connect | ${connect} |
| helo | ${helo} |
| mail | ${mail} | 
| rcpt | ${rcpt} | | Array of email addresses 
| preq | ${data} | | Email body
| postq | | | Not implemented in RC1

***!!!!!!!!!!!!!!! Il faut vraiment le msg_ID***

Let's refine the log in the last example with the IP:port.
```
obj ip4 "localhost" "192.168.1.34";

action "my_action" [
	vsl.DUMP(`/tmp/mail/dump/`),   
	vsl.FACCEPT(),
]

rule connect "check on connect" #{
	condition:  || check_connect.call("localhost"),
    on_success: || my_action(),
    on_failure: || { 
    	vsl.LOG(`Connection from '${source_ip}':'${source_port}' is not allowed.`, "stdout"); 
    	vsl.DENY() 
	},
};
```


## Configuration parameters
Les variables définies par défaut sont accessibles... blahblah  
////////////////////////  
ex. ${time_out_de_je_sais_pas_quoi}

<br>

## Using RHAI for programming complex actions
On top of vSL predefined actions, users can define complex rules through RHAI native language.  
In any case the entry point to interact with the SMTP traffic must be the vSL "rule" function.

```
let my_string = "7x6 = 42";
...

vsl.LOG(`I'm writing this string : ${my_string} into stderr`, "stderr");
```



```
let localhost = "192.168.1.34";

fn my_condition() {
	let my_int = if vsl.IS_CONNECT("localhost") { 42 } else { 0 };
	if (my_int == 42) { 
		true
	} else {
		false
		///////////////// C JUSTE CA ?????
	}
}

fn my_action1() {
    vsl.LOG(`Ok - coming from localhost`, "stdout");
    vsl.CONTINUE()
}

fn my_action2() {
	let admin = "admin@foobar.com";
    vsl.LOG(`Not from localhost. Logging the recipients's list:`, "stderr");
    for rc in rcpt {
      vsl.LOG(`  - ${rc}`, "stdout");
    }
	vsl.BCC(`${admin}`);   
    vsl.CONTINUE()
}

rule rcpt "rcpt_log" #{
	condition:  || my_condition(),
	on_success: || my_action1(),
	on_failure: || my_action2(),
};
```

### Calling external functions
Will be implemented in release candidate 3.

### Shortcuts

If a function has no parameter and there's no computation, || and ( ) can be omitted.
```
fn my_func() {
	...
	vsl.ACCEPT()
}

rule connect "check on connect" #{
	condition:  true,
	on_success: my_func,
	on_failure: vsl.DENY
};
```

But :
```
let boo = 42;
fn my_func(x, y) {
	...
	vsl.ACCEPT()
}

rule connect "check on connect" #{
	condition: || (boo == 42),
	on_success: || my_func(x, y),
	on_failure: vsl.DENY
};

```

 
<br>

**A voir :**


Virer les .rhai et mettre des .vsl :))  
Manque une partie formation, dans le site web :))  
Demander à Mathieu pourquoi c chelou le footer   
Check la sécurité du site et ajouter le bon plug in


//  
//  
Action par défaut en tête de fichier ?  
ex. DEFAULT_ON_SUCCESS = ACCEPT();
//  
//    
// 