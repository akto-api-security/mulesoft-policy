# Steps to run

1. Follow [PDK Prerequisites](https://docs.mulesoft.com/pdk/latest/policies-pdk-prerequisites) on the Mulesoft documentation site, and setup the basic requirements

2. Initialise a new project ```anypoint-cli-v4 pdk policy-project create --name <my-custom-policy>```

3. Copy content of gcl.yaml, lib.rs, Corgo.toml files, and paste these files in your project at respective locations.

4. Compile the project by running ```make build```

5. Publish the policy to mulesoft exchange by running ```make publish```

6. You can now apply this custom policy on your flex gateway api's.
