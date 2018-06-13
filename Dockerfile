FROM library/openjdk
RUN apt-get -y update &&\
    apt-get install -y \
    maven \
    dejagnu \
    postgresql-client \
    less

RUN mkdir /onemodel
WORKDIR /onemodel
COPY . /onemodel

RUN mvn clean package -DskipTests=true

ENV PATH="/onemodel/core/linux/bin:${PATH}"
