> demo

# Structure

## Host

### Basics

1. launch the application and show the __washboard__
2. explain the washboard and displayed __artifacts__
3. do some __inferencing__
4. play with __endpoints__

#### Inferencing

```bash
curl --silent -T ../images/cat.jpg localhost:8078/mobilenetv27/matches | jq
curl -v -T ../images/cat.jpg localhost:8078/mobilenetv27/matches | jq
```

Alternatives are `whales.jpg`, `apple.jpg`, `coffee.jpg`, `hotdog.jpg` &rarr; lunch-time

#### Endpoints

```bash
curl --silent -T ../images/cat.jpg localhost:8078/squeezenetv117/preprocess
```


### Advanced 

1. __scale__ actors 
2. __hot-swap__ an actor

### Hot Swap

1. set `replicas` to 0 for __inferenceapi__
2. select `Start Actor` &rarr; `From File (Hot Reload)`

```bash
/home/cb/dev/wasmcloud/pervaisive/wasmCloudArtefacts/actors/inferenceapi/build/inferenceapi_s.wasm
```

### Plan-B (in case ARM does not work)

* show http config file
* explain how the configuration mechanism works &rarr; __*Link Definitions*__

#### Configuration

1. show `deploy/http_config.json`
2. show `deploy/actor_config.json`

___

## ARM

### Basics

* explain the __context change__
* launch the application and show the __inventory__ in the logs
* explaind the different __architecture__ shown in the inventory
* do some __inferencing__

### Advanced

* show logs
* explain the call chain in the logs


