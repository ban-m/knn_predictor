ECOLIFILE := /glusterfs/ban-m/E_coli_K12_1D_R9.2_SpotON_2/downloads/pass/
ECOLIREF := /glusterfs/ban-m/reference.fasta
MODEL := /home/ban-m/kmer_models/r9.2_180mv_250bps_6mer/template_median68pA.model
SAMFILE := /glusterfs/ban-m/bwamap/mapped.sam
TIMES := 2000
.PHONY: clean

generatedata:
	qsub generatedata.job

knn:
	cargo build --release --bin knn
	qsub knn_chiba.job
	qsub knn_sub.job
	qsub knn_mock.job

setup:
	cp $(ECOLIREF) ./data/reference.fa
	cp $(MODEL) ./data/template.model
	cargo run --release --bin sampreprocess -- $(TIMES) $(ECOLIFILE) $(SAMFILE) > ./data/queries.dat


test:
	cargo build --release --bin knn
	bash knn_test.job
