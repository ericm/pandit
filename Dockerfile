FROM archlinux

COPY target/debug/panditd /panditd

WORKDIR /etc/pandit/services

COPY .pandit.docker.yml /.pandit.yml

ENTRYPOINT [ "/panditd", "--k8s", "-c", "/.pandit.yml" ]