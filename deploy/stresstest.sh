for i in {1..500}
do
   echo "launching $i th request"
   curl -v -X POST 0.0.0.0:8078/model/challenger/index/0 -d '{"tensorType":{"ttype":0},"dimensions":[1,4],"data":[0,0,128,63,0,0,0,64,0,0,64,64,0,0,128,64]}'
done