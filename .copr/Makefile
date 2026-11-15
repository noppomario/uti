# COPR "make_srpm" method: runs as root in a chroot with network access.
# https://docs.pagure.org/copr.copr/user_documentation.html#make-srpm

TOP := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))..

.PHONY: srpm
srpm:
	dnf -y install --setopt=install_weak_deps=False git cargo rpm-build
	$(TOP)/packaging/make-srpm.sh $(outdir)
