FROM archlinux

WORKDIR /

COPY .pandit.docker.yml .pandit.yml

COPY target/debug/pandit /pandit

ENTRYPOINT [ "/pandit", "--docker" ]