FROM debian:stretch

RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y build-essential cmake flex libbison-dev \
    && apt-get clean

CMD ["bash"]
