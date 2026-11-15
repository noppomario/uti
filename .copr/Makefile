# COPR "make_srpm" method: runs as root in a chroot with network access.
# https://docs.pagure.org/copr.copr/user_documentation.html#make-srpm
#
# Builds the latest release tag reachable from the cloned branch, never the
# branch tip: a leaked webhook URL or a merge right after tagging cannot
# publish unreleased code.

TOP := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))..

.PHONY: srpm
srpm:
	dnf -y install --setopt=install_weak_deps=False git cargo rpm-build
	tag=$$(git -c safe.directory='*' -C $(TOP) describe --tags --abbrev=0 --match 'v[0-9]*') && \
	echo "$$tag" | grep -Eqx 'v[0-9]+\.[0-9]+\.[0-9]+' && \
	$(TOP)/packaging/make-srpm.sh $(outdir) "refs/tags/$$tag"
