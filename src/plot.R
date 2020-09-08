setwd("~/work/knn_predictor")
library("tidyverse")


chiba <- read_csv("./result/knn-chiba.csv",col_names = FALSE) %>% rename(refsize = X1,accuracy = X2,k = X3)
sub <-  read_csv("./result/knn-sub.csv",col_names = FALSE) %>% rename(refsize = X1,accuracy = X2,k = X3)
mock <- read_csv("./result/knn-mock.csv",col_names = FALSE) %>% rename(refsize = X1,accuracy = X2,k = X3)

generalplot <- function(g,name){
    pdf(paste0("./pdf/",name,".pdf"))
    plot(g)
    dev.off()
    png(paste0("./png/",name,".png"))
    plot(g)
    dev.off()
}

tileplot <- function(df,name){
    name <- paste0("tileplot_",name)
    g <- df %>% mutate(refsize = as.factor(refsize)) %>% ggplot(mapping = aes(x = k,y= refsize,fill = accuracy)) + geom_tile() +
        labs(title = paste0("headmap of k-NN accuracy:",name),
             x = "k",y = "size of reference(bp)") +  scale_fill_continuous(limits=c(0.5,1), breaks=seq(0.5,1,by=0.05))
    generalplot(g,name)
}

tileplot(chiba,"ChibaNormalFlat1000")
tileplot(sub,"SubHillNormal450")
tileplot(mock,"SubNormalNormal250")

chiba <- chiba %>% nest(-refsize) %>% mutate(data = map(data,function(x){x %>% summarize(max = max(accuracy))})) %>% unnest() %>% mutate(type = "Chiba")
sub <- sub %>% nest(-refsize) %>% mutate(data = map(data,function(x){x %>% summarize(max = max(accuracy))})) %>% unnest() %>% arrange(refsize) %>% mutate(type = "Sub")
mock <- mock %>% nest(-refsize) %>% mutate(data = map(data,function(x){x %>% summarize(max = max(accuracy))})) %>% unnest() %>% arrange(refsize) %>% mutate(type = "mock")

optimal_scores <- bind_rows(chiba,bind_rows(sub,mock)) %>% mutate(type = as.factor(type))

g <- optimal_scores %>% ggplot(mapping = aes(x = refsize, y = max,colour = type)) + geom_line() +
    labs(title = "rel. bet. optimal accuracy and reference size",
         x = "reference size(bp)", y = "optimal accuracy")
generalplot(g,"accuracy_plot")
