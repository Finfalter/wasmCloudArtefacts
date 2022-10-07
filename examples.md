
curl -v -X POST 0.0.0.0:8078/model/challenger/index/0 --data-binary "@data.txt" 

curl -v POST 0.0.0.0:8078/model/identity/index/0 -d '{"tensorType":{"ttype":0},"dimensions":[1,4],"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}'

curl -v POST 0.0.0.0:8078/model/identity/index/0 -d '{"tensorType":{"F32":0},"dimensions":[1,4],"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}'

curl -v POST 0.0.0.0:8078/model/identity/index/0 -d '{"dimensions":[1,4],"valueTypes":["ValueF32"],"flags":0,"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}'
curl -v POST 0.0.0.0:8078/model/plus3/index/0 -d '{"dimensions":[1,4],"valueTypes":["ValueF32"],"flags":0,"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}'
curl -v POST 0.0.0.0:8078/model/mobilenetv27/preprocess/image --data-binary @../providers/mlinference/tests/testdata/images/n04350905.jpg
curl -v POST 0.0.0.0:8078/model/squeezenetv117/preprocess/image --data-binary @../providers/mlinference/tests/testdata/images/n04350905.jpg


curl -v POST 0.0.0.0:8078/identity -d '{"dimensions":[1,4],"valueTypes":["ValueF32"],"flags":0,"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}'
curl -v POST 0.0.0.0:8078/plus3 -d '{"dimensions":[1,4],"valueTypes":["ValueF32"],"flags":0,"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}'
curl -v POST 0.0.0.0:8078/mobilenetv27/preprocess --data-binary @../providers/mlinference/tests/testdata/images/n04350905.jpg
curl -v POST 0.0.0.0:8078/squeezenetv117/preprocess --data-binary @../providers/mlinference/tests/testdata/images/n04350905.jpg

curl -v POST 0.0.0.0:8078/squeezenetv117/matches --data-binary @../providers/mlinference/tests/testdata/images/n04350905.jpg

curl --silent -T ../images/cat.jpg 192.168.178.134:8078/mobilenetv27/matches | jq

curl --silent -T ../images/cat.jpg localhost:8078/mobilenetv27/matches | jq
curl --silent -T ../images/cat.jpg localhost:8078/mobilenetv27/matches | jq
curl --silent -T ../images/4.png localhost:8078/mnistv1/mnist/matches | jq
curl --silent -T ../images/cat_edgetpu.bmp localhost:8078/mobilenetv1tpu | jq
curl --silent -T ../images/cat.jpg localhost:8078/mobilenetv1tpu/matches/rgb8 | jq
