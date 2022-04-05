# rustometer
A little demo showing how to add telemetry to rust and scrape it in prometheus

## GETTING STARTED
To run this demo you will need the following tools:
* docker desktop
* curl
* skaffold

### SKAFFOLD
Skaffold is a tool that builds various Dockerfiles, tags those files based on a revision number and deploys to Kubernetes via helm using the new image tags.

Follow these instructions to install Skaffold:

[https://skaffold.dev/docs/install/#standalone-binary](Install Skaffold)

### DOCKER DESKTOP
Make sure Docker Desktop is configured to run as Kubernetes:
[https://docs.docker.com/desktop/kubernetes/](Docker Desktop Kubernetes Mode)

## RUNNING

### DEPLOY WITH SKAFFOLD

Make sure your kubectl is pointed to your docker desktop kubernetes (this demo will only work on docker desktop)

```bash
skaffold run
```

Check to see if the two pods are running:
```bash
kubectl get pods                                                                          
```

### INCREMENT COUNTER
Use curl to increment the rustometer counter:

```bash
curl http://localhost:8080/count 
```  


### EXECUTE A SEARCH IN PROMETHEUS
Prometheus should be scraping *rustometer* every 5 seconds.

go to the prometheus web interface here: http://localhost:9090/

Execute this query in prometheus ```{__name__!=""}```

You should see several metrics, including the 'counter' metric which was generated by custom instrumentation in rustometer.


