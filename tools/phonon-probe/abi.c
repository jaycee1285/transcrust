#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <time.h>
#include <math.h>
static double seconds(void) {struct timespec t;clock_gettime(CLOCK_MONOTONIC,&t);return t.tv_sec+t.tv_nsec/1e9;}
static void memory(const char *phase) {
    FILE *f=fopen("/proc/self/status","r");char line[256];printf("%s\n",phase);
    if(f){while(fgets(line,sizeof line,f))if(!strncmp(line,"VmRSS:",6)||!strncmp(line,"VmHWM:",6))fputs(line,stdout);fclose(f);}
}
int main(int argc, char **argv) {
    if (argc != 2 && argc != 3) {fprintf(stderr,"usage: abi LIBPHONON2_ENC [PINNED_MODEL_FERMION]\n");return 2;}
    void *lib=dlopen(argv[1], RTLD_NOW|RTLD_LOCAL);
    if (!lib) {fprintf(stderr,"%s\n",dlerror());return 1;}
    int (*abi)(void)=dlsym(lib,"phonon2_cpu_abi_version");
    int (*enc)(void)=dlsym(lib,"phonon2_enc_abi_version");
    int (*kernel)(void)=dlsym(lib,"phonon2_cpu_kernel_in_use");
    unsigned (*features)(void)=dlsym(lib,"phonon2_cpu_features");
    if(!abi||!enc||!kernel||!features)return 1;
    printf("cpu_abi=%d encoder_abi=%d kernel=%d features=0x%x\n",abi(),enc(),kernel(),features());
    printf("wire constructor=%s encoder forward=%s\n",dlsym(lib,"phonon2_cpu_create_onedot_wire")?"available":"missing",dlsym(lib,"phonon2_enc_forward")?"available":"missing");
    if(argc==3) {
        /* Only the pinned English 2.5 artifact's FIRST matrix, not model loading.
         * The caller verifies the whole container hash before running this.
         * Its first record is [4096,1024] five_value; do not generalize this
         * small probe's header parser into an application loader. */
        FILE *f=fopen(argv[2],"rb");if(!f)return 1;
        uint64_t hlen=0;if(fread(&hlen,8,1,f)!=1||hlen>1024*1024)return 1;
        char *head=calloc(hlen+1,1);if(!head||fread(head,1,hlen,f)!=hlen)return 1;
        const char *prefix="{\"format\": \"fermion-five-value-parakeet-v1\", \"index\": [{\"n\": \"encoder.layers.0.feed_forward1.linear1\", \"k\": \"five_value\", \"shape\": [4096, 1024], \"b\": ";
        if(strncmp(head,prefix,strlen(prefix))){fprintf(stderr,"unexpected first record\n");return 1;}
        size_t n=strtoull(head+strlen(prefix),NULL,10);if(n!=1115113)return 1;
        void *wire=malloc(n);if(!wire||fread(wire,1,n,f)!=n)return 1;fclose(f);free(head);
        void *(*create)(int,int,const void*,size_t)=dlsym(lib,"phonon2_cpu_create_onedot_wire");
        void (*destroy)(void*)=dlsym(lib,"phonon2_cpu_destroy");
        void (*threads)(int)=dlsym(lib,"phonon2_cpu_set_threads");
        int (*gemm)(void*,const void*,int,void*,int)=dlsym(lib,"phonon2_cpu_gemm");
        if(!create||!destroy||!threads||!gemm)return 1;
        threads(4);memory("before matrix");double load=seconds();void *matrix=create(4096,1024,wire,n);load=seconds()-load;
        if(!matrix)return 1;free(wire);memory("after matrix (source freed)");
        float *x=malloc(80*1024*sizeof(float)),*y=malloc(80*4096*sizeof(float));if(!x||!y)return 1;
        for(int i=0;i<80*1024;i++)x[i]=sinf(i*.17f)*.1f;
        if(gemm(matrix,x,80,y,4096))return 1;
        double begin=seconds();for(int i=0;i<20;i++)if(gemm(matrix,x,80,y,4096))return 1;
        printf("FIRST-MATRIX ONLY: build %.3f ms; mean warm GEMM %.3f ms (M=80,K=1024,N=4096,4 threads,20 repeats)\n",load*1e3,(seconds()-begin)*1e3/20);
        double checksum=0;for(int i=0;i<80*4096;i++){if(!isfinite(y[i]))return 1;checksum+=y[i];}
        printf("finite output checksum %.9g; NOT an upstream numerical-parity check\n",checksum);memory("after GEMM");
        free(x);free(y);destroy(matrix);
    }
    dlclose(lib);return 0;
}
