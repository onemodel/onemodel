FROM library/openjdk
RUN apt-get -y update &&\
    apt-get install -y \
    maven \
    dejagnu \
    postgresql-client

RUN mkdir /onemodel
WORKDIR /onemodel
COPY . /onemodel

RUN mvn clean package -DskipTests=true

