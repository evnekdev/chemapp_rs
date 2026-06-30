
/* Note that this program contains target calculations,
   which cannot be performed with the 'light' version of ChemApp */ 

/* 

   Please also read the notes regarding the use of ChemApp with C/C++,
   which are contained in a separate file.

   Please direct any questions about this program or the results it
   produces on your machine to:

   GTT-Technologies
   Kaiserstrasse 100
   52134 Herzogenrath
   Germany

   Phone:  +49-2407-59533
   Fax:    +49-2407-59661
   E-mail: support@gtt-technologies.de
   WWW:    http://www.gtt-technologies.de/

*/

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "cacint.h"

/* ---------------------------------------------------------------------
 * Func    : abortprog
 * ---------------------------------------------------------------------
 * Subject : If a ChemApp error occurs, this function reports the error
             number and the routine it occurred in before it exits the
             program.
 *
 * ---------------------------------------------------------------------
 */
void abortprog(int lineno, char sr_name[10], LI error_no)
{

    fprintf(stdout,"\nChemApp error no. %li occurred when calling %s.\n"
	    "Aborting on line %i of %s .\n", error_no, sr_name, 
        lineno, __FILE__);
    exit(error_no);
}    
    



/* ---------------------------------------------------------------------
 * Func    : main
 * ---------------------------------------------------------------------
 * Subject : Main body of program
 *
 * ---------------------------------------------------------------------
 */
int main()
{
  
  /* Declaration of variables to be used in the program */

  /* Every integer variable that is to be used in an invocation of a 
     ChemApp subroutine has to be of type "LI". "LI" is defined in 
     cacint.h and is usually an abbreviation for "long int". Similarly,
     every real variable for use with ChemApp subroutines has to be of
     type "DB", which is usually an abbreviation for "double" */
  

  /* Declarations of integer variables for use with ChemApp: */
  
  /* An all-purpose integer variable */
  LI lint,
    
    /* Integer variables which hold the index number of phases, phase
       constituents, and sublattices */
    indexp, indexc, indexl,
    
    
    /* The standard error code return variable */
    noerr,  
    
    /* A variable that is set to the version number of ChemApp */
    cavers,
    
    /* This variable is used in conjunction with tqlite to determine
       whether the program is linked to the regular or the 'light'
       version of ChemApp. In the latter case, target calculations are
       left out. */
    islite,

    /* A number of integer variables to return the information from tqsize */
    lia,lib,lic,lid,lie,lif,lig,lih,lii,lij,lik,

    /* A similar set of integer variables to return the information from
       tqused */
    ldia,ldib,ldic,ldid,ldie,ldif,ldig,ldih,ldii,ldij,ldik,

    /* An integer that will contain the index number of a condition set */
    numcon,
    
    /* An integer that will contain the number of phases in the
       data-file loaded */
    nphase,

    /* An integer that will hold the number of phase constituents in
       the gas phase */
    npcgas,

    /* An integer that will contain the number of system components */
    nscom,

    /* Integers that will contain the number of sublattices, and the
       number of sublattice contituents of a particular sublattice */
    nsl, nslcon,

    /* Two integers used for one-dimensional phase mapping calculations, one
       to indicate whether we need to make more calls to tqmap/tqmapl */
    icont,

    /* And another one to keep track of the number of results we
       obtained */
    resno,

    /* An integer that holds the FORTRAN unit number under which the 
       thermochemical data-files will be opened */
    unitno,

    /* An integer that holds the FORTRAN unit number to which the
       ChemApp error messages are written by default */
    errorunit,
    
    /* Variables to store information from various transparent file header 
       fields */
    tfhver, tfhvnw[3], tfhvnr[3], tfhdtc[6], tfhdte[6],

    /* These integers will be used to store the HASP dongle id, plus the
       ChemApp expiration month and year */
    haspid, edmon, edyear;

  /* Declarations of real variables for use with ChemApp: */

  /* An all-purpose real variable */
  DB d1, 

    /* An array of two real variables... */
    darray2[2],
      
    /* ...and one of 30 */
    darray30[30], 
      
    /* ...and one for which memory will be allocated dynamically to be
       just of the right size */
    *darray,

    /* A variable to hold the molecular mass of a system component
       (see tqstsc) or a phase constituent (see tqstpc) */
    wmass,

    /* This particular array of two real variables will contain the
       temperature and pressure of a stream to be defined
       with tqsttp */
    tp[2];


  /* Declarations of string variables for use with ChemApp: */
    
    
  /* Every "name" used in conjuction with ChemApp, whether it is the
     name of a phase, a system component, a phase constituent, or a
     stream, can be up to 24 characters in length (Exceptions are the
     subroutine tqerr, as well as the subroutines tqgtrh, tqgtid,
     tqgtnm, tqgtpi, and tqwstr, which all pass or return strings
     longer than 24 characters). This means that in C/C++, these
     strings have to be 25 characters long, due to the trailing null
     character (\0). TQSTRLEN is defined in cacint.h to be 25.  Thus,
     if you define all your strings to be used with ChemApp using
     TQSTRLEN, you can be sure that they will have the right
     length. Another way to do this is to use the macro "TQSTRING(x)",
     as defined in cacint.h, which expands to "char x[TQSTRLEN]". */

  /* an all-purpose static string for ChemApp */
  char dstr[TQSTRLEN],
      
    /* A pointer to a string, before it can be used, sufficient memory
       has to be allocated */
    *dstrptr,
    
    /* Character strings that will hold names of phases, phase
       constituents, system components, mixture model names, as well as
       sublattatices and their constituents */

    pname[TQSTRLEN], cname[TQSTRLEN], mname[TQSTRLEN],

    /* The following variable is an array of 4 strings. It will be
       used with tqcsc, which is the only ChemApp subroutine that
       takes a character array as input. Please note that the last
       string element of such an array for use with tqcsc must be an
       extra empty string, this is due to the peculiarities of
       exchanging strings between FORTRAN and C code. This is why we
       declare the following array to have 4 elements, although we
       will only have 3 system components. */
    newsc[4][TQSTRLEN];

  /* Using the macro TQSTRING to declare a string variable for
     ChemApp*/
  TQSTRING(dstr2);

  /* The next three character strings are used to hold information
     about the copy of ChemApp used. */
  char id[255], name[80], pid[TQSTRLEN];

  /* Variables to store information from various transparent file header 
     fields */
  char tfhid[255], tfhusr[80],tfhnwp[40],tfhnrp[40],tfhrem[80];


  /* This character variable will be used to store the HASP dongle type */
  char haspt[TQSTRLEN];


  /* Declaration of various other variables: */
  int i;
  
  
  /* Print the program title and its ID */
  printf("\nThis is cademo1, a C program to test ChemApp and its"
	 " C Interface.\n\n");
  

  
  /* Allocate memory for the dynamic string variable dstrptr. By using
     TQSTRLEN, we make sure it has the right length. */
  dstrptr = malloc(TQSTRLEN);
  
  /* Initialise ChemApp */ 
  tqini(&noerr);
  
  /* After all following invocations of ChemApp subroutines, the error
     variable will be checked for a non-zero value, in which case an 
     error has occurred and the program is aborted */
  if (noerr) abortprog(__LINE__,"tqini",noerr);


  /* Print the copyright string. The way to do this is to first
     call the subroutine tqcprt, which copies the copyright string into
     the ChemApp-internal error buffer string: */
  tqcprt(&noerr);
  if (noerr) abortprog(__LINE__,"tqcprt",noerr);

  /* Then do what is usually done to retrieve an error message: 
     call tqerr to retrieve it. The variable TQERRMSG is declared in 
     cacint.c as TQERRMSG[3][80] for this purpose:*/

  tqerr((CHP)TQERRMSG,&noerr);
  /* Note that an explicit typecast has been used in the above call
     to tqerr to avoid compiler warnings about TQERRMSG being an
     incompatible pointer type ("CHP" is declared in cacint.h to be
     an abbreviation for "char*") */

  if (noerr) abortprog(__LINE__,"tqerr",noerr);

  /* To print out the message, loop over the three strings of 
     TQERRMSG: */
  for(i=0;i<3;i++) 
    printf("%s\n",TQERRMSG[i]);



  /* Get version number of ChemApp*/
  tqvers(&cavers,&noerr);
  if (noerr) abortprog(__LINE__,"tqvers",noerr);
  printf("\nChemApp version is: %li\n\n", cavers);
    
  /* Get array sizes */
  tqsize(&lia,&lib,&lic,&lid,&lie,&lif,&lig,&lih,&lii,&lij,&lik,&noerr);
  if (noerr) abortprog(__LINE__,"tqsize",noerr);
  printf("Internal array sizes of this version of ChemApp,\n"
	 "maximum number of:\n"
	 "constituents: %li\n"
	 "system components: %li\n"
	 "mixture phases: %li\n"
	 "excess Gibbs energy coefficients for "
	 "a mixture phase: %li\n"
	 "excess magnetic coefficients for "
	 "a mixture phase: %li\n"
	 "sublattices for a mixture phase: %li\n"
	 "constituents of a sublattice: %li\n"
	 "oxide constituents of a phase described by the "
	 "Gaye-Kapoor-Frohberg\n"
	 " or modified quasichemical formalisms: %li\n"
	 "Gibbs energy/heat capacity equations for "
	 "a constituent: %li\n"
	 "Gibbs energy/heat capacity equations: %li\n"
	 "constituents with P,T-dependent "
	 "molar volumes: %li\n\n",
	 lia,lib,lic,lid,lie,lif,lig,lih,lii,lij,lik);
    



  /* Get value for input/output option */

  /* Determine which FORTRAN unit is used by default by tqrfil for
     reading the thermochemical data-file */
  tqgio("FILE", &unitno, &noerr);
  if (noerr) abortprog(__LINE__,"tqgio",noerr);
  printf("The thermochemical data will be read from the file "
	 "associated with unit %li\n\n", unitno);


  /* Open data-file for reading */     
  /* Note that tqopna is used to open the thermochemical data-file,
     instead of a standard C library routine like fopen. The reason
     is that it is necessary to associate the data-file with a
     particular FORTRAN unit number (10 by default, see above) so
     that the FORTRAN subroutine tqrfil can read from the correct
     file */
  tqopna("cosi.dat",unitno,&noerr);
  if (noerr) abortprog(__LINE__,"tqopna",noerr);

  /* Read data-file */
  tqrfil(&noerr);
  if (noerr) abortprog(__LINE__,"tqrfil",noerr);

  /* Close data-file */
  /* Again, the routine for closing the data-file is not a standard
     C library routine like fclose, but a special one to make sure the
     file that was previously opened under the unit number specified
     is closed */
  tqclos(unitno,&noerr);
  if (noerr) abortprog(__LINE__,"tqclos",noerr);

  /*C*# label(tqused) */

  /* tqused is used to find out to what extent the thermochemical
     data storage space in the internal arrays of ChemApp (as
     reported by tqsize) are occupied by the currently loaded
     thermochemical system. */
  tqused(&ldia,&ldib,&ldic,&ldid,&ldie,&ldif,&ldig,&ldih,&ldii,
	 &ldij,&ldik,&noerr);
  if (noerr) abortprog(__LINE__,"tqused",noerr);

  /* As an example, check how many Gibbs energy/heat capacity
     equations are currently being used. */
  printf("Actual number of Gibbs energy/heat capacity equations "
	 "used (tqused): %li\n", ldij);
  printf("Maximum number available in this version of ChemApp "
	 "(tqsize): %li\n", lij);



  /* Get system units */
  tqgsu("Pressure",dstr,&noerr);
  if (noerr) abortprog(__LINE__,"tqgsu",noerr);
  printf("Pressure unit: %s\n", dstr);

  tqgsu("Volume",dstr,&noerr);
  if (noerr) abortprog(__LINE__,"tqgsu",noerr);
  printf("Volume unit: %s\n", dstr);

  tqgsu("Temperature",dstr,&noerr);
  if (noerr) abortprog(__LINE__,"tqgsu",noerr);
  printf("Temperature unit: %s\n", dstr);

  tqgsu("Energy",dstr,&noerr);
  if (noerr) abortprog(__LINE__,"tqgsu",noerr);
  printf("Energy unit: %s\n", dstr);

  /* Change system unit */
  /* Here we change the unit for the quantity "amount" to gram,
     mainly so that when we call tqstsc further down, we get the
     molecular mass expressed in the unit g/mol */
  tqcsu("Amount","gram", &noerr);
  if (noerr) abortprog(__LINE__,"tqcsu",noerr);
  printf("Amount unit set to gram\n\n");



  /* Get number of system components */
  tqnosc(&nscom,&noerr);
  if (noerr) abortprog(__LINE__,"tqnosc",noerr);
  printf("Number of system components: %li\n\n", nscom);

  /* Get stoichiometry of system component */
  /* Note that the array used (darray30) is much bigger than needed,
     which doesn't matter to ChemApp, as long as it is big enough. */
  tqstsc(1,darray30,&wmass,&noerr);
  if (noerr) abortprog(__LINE__,"tqstsc",noerr);

  /* Here the array elements are printed, which describe the
     stoichiometry of the system component. This particular piece of
     information is not too useful though, since the array contains
     all zeros, except for a "1.0" at the position of the system
     component we specified */
  printf("Stoichiometry of system component 1: ");
  for (i=1;i<=nscom;i++) 
    printf("%f ", darray30[i-1]);

  /* Had we not changed the amount unit from the default value "mol"
     to "gram", we would receive a value of 1.0 for wmass. The reason
     is that the molecular mass is always output using the unit
     [current amount unit]/mol. If the current amount unit happens to
     be mol, the value for wmass will be 1.0 */
  printf("\nMolecular mass in g/mol: %f\n", wmass);


  /* Get name of system component #1 */
  tqgnsc(1,dstr,&noerr);
  if (noerr) abortprog(__LINE__,"tqgnsc",noerr);
  printf("Name of system component 1: %s\n", dstr);
    
  /* Get index number of system component */
  /* This is the reverse of the above, using the name of the 
     system component to get its index number */
  tqinsc(dstr, &lint, &noerr);
  if (noerr) abortprog(__LINE__,"tqinsc",noerr);
  printf("Index number of system component %s is: %li\n\n",dstr,lint);



  /* Change system components */
  /* tqcsc is the only ChemApp subroutine which takes an array of
     strings as input. The array contains as many strings as there
     are system components in the data-file, plus an extra one is
     needed when programming in C, which has to be empty. */
    
  /* The system components are changed to SiO, SiC, and CO. */
  strcpy(newsc[0], "SiO"); 
  strcpy(newsc[1], "SiC"); 
  strcpy(newsc[2], "CO"); 

  /* Don't forget: The last string of the string array that will be
     passed to tqcsc has to be empty! */
  strcpy(newsc[3], ""); 

  tqcsc((CHP)newsc,&noerr); 
  /* Note that an explicit typecast has been used in the above call
     to tqcsc to avoid compiler warnings about newsc being an
     incompatible pointer type */
    
  if (noerr)abortprog(__LINE__,"tqcsc",noerr); 
  printf("System components changed to SiO-SiC-CO.\n");
    

  /* Print names of new system components */
  for (lint = 1; lint <= nscom; lint++) {
    tqgnsc(lint,dstr,&noerr);
    if (noerr) abortprog(__LINE__,"tqgnsc",noerr);
    printf("Name of new system component %li: %s\n", lint, dstr);
  }

  /* Get stoichiometry of system component #1 */
  tqstsc(1,darray30,&d1,&noerr);
  if (noerr) abortprog(__LINE__,"tqstsc",noerr);

  /* Note that the stoichiometry of the system component #1 does not
     change, as compared to the first call to tqstsc before the
     system components have been altered: */
  printf("Stoichiometry of system component 1: ");
  for (i=1;i<=nscom;i++) 
    printf("%f ", darray30[i-1]);
  /* What changes of course is the molecular mass: */
  printf("\nMolecular mass in g/mol: %f\n\n", d1);



   
  /* Get number of phases */
  tqnop(&nphase,&noerr);
  if (noerr) abortprog(__LINE__,"tqnop",noerr);
  printf("Number of phases: %li\n", nphase);


  /* Get name of phase #1 */
  tqgnp(1,dstr,&noerr);
  if (noerr) abortprog(__LINE__,"tqgnp",noerr);
  printf("Name of phase 1: %s\n", dstr);
    

  /* Get index number of phase */
  /* This is the reverse of the above, using the name of the 
     phase to get its index number */
  tqinp(dstr,&lint,&noerr);
  if (noerr) abortprog(__LINE__,"tqinp",noerr);
  printf("Index number of phase %s is: %li\n",dstr,lint);


  /* Get model name of phase */
  tqmodl(1, dstr, &noerr);
  if (noerr) abortprog(__LINE__,"tqmodl",noerr);
  printf("Model name of phase 1: %s\n",dstr);

  /* Whereas above we asked for the model name of the first phase in
     the data-file, which is normally a mixture phase (gas phase),
     we now ask for the model name of the last phase in the
     data-file (nphase, as determined by a call to tqnop). This is
     typically a stoichiometric condensed phase, in which case the
     string "PURE" is returned as the model name*/
  tqmodl(nphase, dstr, &noerr);
  if (noerr) abortprog(__LINE__,"tqmodl",noerr);
  printf("Model name of phase %li: %s\n\n",nphase,dstr);


  /* Get number of phase constituents of phase #1 */
  tqnopc(1, &npcgas, &noerr);
  if (noerr) abortprog(__LINE__,"tqnopc",noerr);
  printf("Number of phase constituents in phase 1: %li\n", npcgas);

  /* Get name of phase constituent */
  tqgnpc(1, 1, dstr, &noerr);
  if (noerr) abortprog(__LINE__,"tqgnpc",noerr);
  printf("Name of phase constituent 1 of phase 1: %s\n", dstr);


  /* Get index number of phase constituent */
  /* This is the reverse of the above, using the name of the phase
     constituent to get its index number */
  tqinpc(dstr, 1, &lint, &noerr);
  if (noerr) abortprog(__LINE__,"tqinpc",noerr);
  printf("Index number of phase constituent %s is: %li\n",dstr,lint);

  /* Check whether the phase constituent can be used as incoming
     species (note that there are only few cases, and only for
     certain mixture phase models, where this might not be
     permitted)*/
  tqpcis(1, 1, &lint, &noerr);
  if (noerr) abortprog(__LINE__,"tqpcis",noerr);
  if (lint == 0)
    printf("Usage of %s as incoming species is not permitted.\n", 
	   dstr);
  else
    printf("Usage of %s as incoming species is o.k.\n", dstr);

  /* Get stoichiometry of phase constituent */
  /* tqstpc is a subroutine similar to tqstsc, except that with
     tqstpc, the returned stoichiometry array contains valuable
     information. This stoichiometry array represents the
     stoichiometry matrix for a phase constituent, as described in
     the ChemApp manual*/
  tqstpc(1, 1, darray30, &wmass, &noerr);
  if (noerr) abortprog(__LINE__,"tqstpc",noerr);
  printf("Stoichiometry of phase constituent 1 in phase 1 is: ");
  for (i=1;i<=nscom;i++) 
    printf("%f ", darray30[i-1]);
  printf("\nMolecular mass: %f\n\n", wmass);


  /* The system components are changed back to the original set of
     C, O, and Si, since we don't need the modified set of system
     components (SiO, SiC, and CO) any longer. */

  strcpy(newsc[0], "C"); 
  strcpy(newsc[1], "O"); 
  strcpy(newsc[2], "Si"); 
  strcpy(newsc[3], ""); 

  tqcsc((CHP)newsc,&noerr); 
  if (noerr)abortprog(__LINE__,"tqcsc",noerr); 
  printf("System components changed back to C-O-Si.\n");

    
  /* Changing the status of phases and phase constituents */
  /* Change status of phase #1 */
  tqcsp(1, "eliminated", &noerr);
  if (noerr) abortprog(__LINE__,"tqcsp",noerr);
  printf("Status of phase 1 set to 'eliminated'\n");

  /* Get status of phase #1 */
  tqgsp(1, dstr, &noerr);
  if (noerr) abortprog(__LINE__,"tqgsp",noerr);
  printf("Status of phase 1 is: %s \n", dstr);

  /* Change status of phase #1 */
  tqcsp(1, "entered", &noerr);
  if (noerr) abortprog(__LINE__,"tqcsp",noerr);
  printf("Status of phase 1 set to 'entered'\n");

  /* Get status of phase #1 */
  tqgsp(1, dstr, &noerr);
  if (noerr) abortprog(__LINE__,"tqgsp",noerr);
  printf("Status of phase 1 is: %s \n", dstr);


    
  /* Change status of phase constituent #1 of phase #1 */
  tqcspc(1, 1, "dormant", &noerr);
  if (noerr) abortprog(__LINE__,"tqcspc",noerr);
  printf("Status of constituent 1 of phase 1 set to 'dormant'\n");

  /* Get status of phase constituent #1 of phase #1 */
  tqgspc(1, 1, dstr, &noerr);
  if (noerr) abortprog(__LINE__,"tqgspc",noerr);
  printf("Status of constituent 1 of phase 1 is: %s \n", dstr);

  /* Change status of phase constituent #1 of phase #1 */
  tqcspc(1, 1, "entered", &noerr);
  if (noerr) abortprog(__LINE__,"tqcspc",noerr);
  printf("Status of constituent 1 of phase 1 set to 'entered'\n");

  /* Get status of phase constituent #1 of phase #1 */
  tqgspc(1, 1, dstr, &noerr);
  if (noerr) abortprog(__LINE__,"tqgspc",noerr);
  printf("Status of constituent 1 of phase 1 is: %s \n\n", dstr);


  printf("\n\nCalculations using global conditions:\n\n");

  /* Set equilibrium conditions */  

  /* When setting equilibrium conditions with tqsetc, the currently
     set units for the various quantities (pressure, amount, etc.)
     apply. Thus, in the following call, the incoming amount ("ia")
     of phase constituent #4 of phase #1 is set to 1.0 gram. */
  tqsetc("ia ", 1, 4, 1.0,&numcon,&noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);

    
  /* Change system unit */
  /* Now we change the amount unit back to "mol". This does not
     affect any previously set values. The amount of phase
     constituent #4 of phase #1 we set above is still 1.0 gram, and
     is not changed to 1.0 mol! */
  tqcsu("Amount","mol", &noerr);
  if (noerr) abortprog(__LINE__,"tqcsu",noerr);
  printf("Amount unit set to mol\n\n");


  /* Since we changed the amount unit to "mol", the call to tqsetc
     below inputs 3.0 mol of constituent #12 of phase #1 */
  tqsetc("ia ", 1, 12, 3.0,&numcon,&noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);

  /* Now we input 8 mol of constituent #8 of phase #1 */
  tqsetc("ia ", 1, 8, 2.0,&numcon,&noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);
    
  /* Remove equilibrium condition */
  /* This call to tqremc demonstrates how the parameter "numcon" is
     used, which is returned with every call to tqsetc: The
     previously defined condition is removed again */
  tqremc(numcon, &noerr);
  if (noerr) abortprog(__LINE__,"tqremc",noerr);
    
  /* Set temperature */
  /* Here tqsetc is used to enter the temperature. Since we didn't
     change the default temperature unit, it is still Kelvin ("K") */
  tqsetc("t ", 0, 0, 1800.0,&numcon,&noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);


  /* Change limit of target variables */
  /* The next call demonstrates how tqclim is used. Normally though
     calls to tqclim are only necessary under special circumstances */
  tqclim("plow", 1e-49, &noerr);
  if (noerr) abortprog(__LINE__,"tqclim",noerr);


  /* Display present settings */
  /* tqshow outputs the conditions which are active at this point */
  printf("\n\nCurrently active conditions:\n");
  printf("\n\n**** Begin output table produced by tqshow\n");

  /* Depending on the type of ChemApp object code used
     (e.g. statically linked OBJ code vs. dynamic link library), and
     possibly also depending on the compiler/platform/operating
     system used, it might be necessary to use fflush(NULL) to flush
     the output buffers in order to synchronise the output of the
     standard C calls and the output through FORTRAN units: */
  fflush(NULL);
  tqshow(&noerr);
  fflush(NULL);
  printf("\n**** End output table produced by tqshow\n\n\n");
  if (noerr) abortprog(__LINE__,"tqshow",noerr);
  printf("\n\n");



  /* Calculate equilibrium */                   
  /* A simple call to tqce or tqcel is sufficient to calculate the
     equilibrium. All input parameters to tqce and tqcel are only
     significant if target calculations are performed. */
  darray2[0]=0.0;
  tqce(" ",0,0,darray2,&noerr); 
  if (noerr) abortprog(__LINE__,"tqce",noerr);


  /* Calculate and list equilibrium */                
  /* tqcel calculates the equilibrium just like tqce, with the only
     difference that a ChemSage-type result table is written to the
     file/unit associated with 'LIST' (see tqgio/tqcio)*/
  printf("\n\n**** Begin output table produced by tqcel\n");
  fflush(NULL);
  tqcel(" ",0,0,darray2,&noerr); 
  fflush(NULL);
  printf("\n**** End output table produced by tqcel\n\n\n");
  if (noerr) abortprog(__LINE__,"tqcel",noerr);



  /* Once an equilibrium has been calculated using tqce/tqcel,
     subsequent calculations can be performed using tqcen/tqcenl,
     which speeds up the equilibrium calculation by taking results
     from the previous calculation as initial estimates */
  tqsetc("t ", 0, 0, 1850.0,&numcon,&noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);
         
  tqcen(" ",0,0,darray2,&noerr); 
  if (noerr) abortprog(__LINE__,"tqcen",noerr);

  tqsetc("t ", 0, 0, 1900.0,&numcon,&noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);
         
  /* Similar to tqcel, tqcenl also provides a ChemSage output table */  
  printf("\n\n**** Begin output table produced by tqcenl\n");
  fflush(NULL);
  tqcenl(" ",0,0,darray2,&noerr); 
  fflush(NULL);
  printf("\n**** End output table produced by tqcenl\n\n\n");
  if (noerr) abortprog(__LINE__,"tqcenl",noerr);


  /* Change value of input/output option */
  /* Redirect the output from tqcel to a file using tqcio with the 
     option "LIST"*/
  /* The following call to tqcio will redirect the output of ChemApp
     subroutines like tqcel and tqshow to the FORTRAN unit number
     21. Note that only certain values for the unit number are
     permitted (see the ChemApp manual entry for tqcio) */
  tqcio("LIST", 21, &noerr);
  if (noerr) abortprog(__LINE__,"tqcio",noerr);

  /* Note that at this point, no file has been associated with unit
     number 21 yet! This is done with the subsequent call to tqopen:*/
  tqopen("result",21,&noerr);
  if (noerr) abortprog(__LINE__,"tqopen",noerr);
  printf("Output of next call to tqcel will be written to "
	 "file \"result\"\n");

  /* tqwstr can be used to write user-defined text to the unit numbers
     associated with "LIST" and "ERROR". This subroutine is especially
     useful for non-FORTRAN programs. */
  tqwstr("LIST", "Output from tqcel (ChemSage result table):", &noerr);
  if (noerr) abortprog(__LINE__,"tqwstr",noerr);

  /* Now the ChemSage-type result table output by the following
     routine is written to the file "result": */
  tqcel(" ",0,0,darray2,&noerr); 
  if (noerr) abortprog(__LINE__,"tqcel",noerr);


  /* Close the file associated with unit number 21 again... */
  tqclos(21,&noerr);
  if (noerr) abortprog(__LINE__,"tqclos",noerr);

  /* ...and redirect the output of routines like tqcel and tqshow
     back to unit number 6, which is the default and associated with
     stdout: */
  tqcio("LIST", 6, &noerr);
  if (noerr) abortprog(__LINE__,"tqcio",noerr);
  printf("Output of subsequent calls to tqcel and tqshow will go "
	 "to stdout again\n\n");



  /* Get and display results */

  /* First, we get the fraction of the first phase constituent of
     the first phase. We get also its name with tqgnpc, this time we
     use the string variable "dstr2" which we declared using the
     macro "TQSTRING". Since the current amount unit is "mol", we
     get the "mole fraction" (as opposed to "mass fraction") */

  tqgnsc(1, dstr2, &noerr);
  if (noerr) abortprog(__LINE__,"tqgnsc",noerr);
  tqgetr("xp ", 1, 1, &d1, &noerr); 
  if (noerr) abortprog(__LINE__,"tqgetr",noerr); 
  printf("Mole fraction of system component %s in the GAS phase: %f\n\n", 
	 dstr2, d1);



  /* Now get the equilibrium amount of the same constituent */
  tqgetr("a ",1,1,&d1,&noerr); 
  if (noerr) abortprog(__LINE__,"tqgetr",noerr); 
  printf("Equilibrium amount of %s in the GAS phase: %f\n\n", dstr2, d1);

  /* Note that the result is zero, because the gas phase is not
     considered stable. The reason is that we are using the default
     ambient pressure of 1 bar for the calculations, and from the
     result table produced with the previous call to tqcel or the
     subsequent call to tqgetr we see that the activity of the gas
     phase is less than unity, which means it is not considered
     stable, and thus the equilibrium amounts of all its phase
     constituents are zero. */

  /* Using the option "ac", tqgetr retrieves activities, in this case the
     activity (fugacity) of the gas phase */
  tqgetr("ac ",1, 0, &d1, &noerr); 
  if (noerr) abortprog(__LINE__,"tqgetr",noerr); 
  printf("Activity of the GAS phase: %f\n\n", d1);


    
  /* Now tqgetr is used in a way that causes it to return an array. In
     this case, we are going to retrieve an array that contains the
     fugacities of all constituents of the gas phase. First, we need
     an array that has enough room for all the results.*/

  /* With an earlier call to tqnopc we determined the number of
     constituents in the gas phase and stored it in npcgas. Thus we
     can use this number to allocate enough memory for the array:*/
   
  if (!(darray = calloc(npcgas, sizeof(DB)))) {
    printf("Not enough memory available for dynamic allocation!\n");
    exit(1);
  }


  /* Get the fugacities:*/
  tqgetr("ac", 1, -1, darray, &noerr);
  if (noerr) abortprog(__LINE__,"tqgetr",noerr); 
    
  /* Print them out, together with the names of the constituents
     (note that these values correspond to the last column of the
     result table previously output with tqcel):*/
    
  printf("\nFugacities of all gas phase constituents:\n");

  /* Note that the 0th element of the array contains the fugacity of
     the 1st constituent of the gas phase. That's why the call to
     tqgnpc has to use "lint+1" instead of "lint"! */
  for (lint = 0; lint < npcgas; lint++) {

    tqgnpc(1, lint+1, dstr, &noerr);
    if (noerr) abortprog(__LINE__,"tqgnpc",noerr); 
      
    printf("%5s: %g\n", dstr, darray[lint]);
  }

  printf("\n\n");

  /* Recycle the memory */
  free(darray);


  /* Get thermodynamic data of a phase constituent */
  tqgdpc("G", 1, 1, &d1, &noerr);
  if (noerr) abortprog(__LINE__,"tqgdpc",noerr);
  printf("The (dimensionless) value of G for "
	 "constituent 1 of phase 1 is: %f\n\n\n", d1);


  /* Note that when retrieving CP, H, S, or G using tqgdpc, the
     values are returned "dimensionless", which means they might have
     to be multiplied by R*T. Since the default amount unit is mol,
     results are returned for 1 mol. Note also that care has to be
     taken if a temperature unit different from Kelvin has been
     used. Refer to the manual entry of tqgdpc for a more detailed
     example. */


  /* Check if we are working with the 'light' version.
     If we do, omit the following target calculation(s). */
  tqlite(&islite, &noerr);
  if (islite) 
    {
      printf("*** Target calculations have been omitted here,\n"
             "*** since they are not possible with the\n"
             "*** 'light' version of ChemApp.\n\n");
    }
  else 
    {


  /* Perform a target calculation. This example demonstrates how to 
     have ChemApp find the temperature at which a certain phase becomes
     stable. */

  /* We want to have ChemApp calculate the temperature at which the
     phase "SiO2(liquid)" becomes stable. First, determine the index
     number of this phase (note that it is allowed to abbreviate its name,
     as long as it is unambiguous):*/
  tqinp("SiO2(liq", &lint, &noerr);
  if (noerr) abortprog(__LINE__,"tqinp",noerr);
 
  /* Next, define the formation target :*/
  tqsetc("a", lint, 0, 0.0, &numcon, &noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);

  /* Then call tqcel to calculate the equilibrium, and pass it the
     information about the target variable (we want ChemApp to vary
     the _temperature_ until the phase is stable): */

  printf("Performing a phase target calculation\n"
	 "Phase target: formation of SiO2(liquid)\n"
	 "Target variable: Temperature\n");

  darray2[0] = 2000.0;
  printf("\n\n**** Begin output table produced by tqcel\n");
  fflush(NULL);
  tqcel("t",0,0,darray2,&noerr); 
  fflush(NULL);
  printf("\n**** End output table produced by tqcel\n\n\n");

  if (noerr) abortprog(__LINE__,"tqcel",noerr);
  darray2[0] = 0.0;

  /* Note that in the resulting ChemSage table, the temperature is
     marked with an asterisk, meaning that it has been calculated by
     ChemApp, and the activity of SiO2(liquid) is unity, whereas its
     equilibrium amount is zero, meaning it just forms at this
     temperature*/
    
  /* Retrieve the calculated temperature:*/
  tqgetr("t", 0, 0, &d1, &noerr);
  if (noerr) abortprog(__LINE__,"tqgetr",noerr); 
  printf("Calculated formation temperature of SiO2(liquid): %g\n\n", d1);

	   

  /* One-dimensional phase mapping calculations. The example below
     demonstrates how to locate all phase transitions in a given
     temperature range. */

  /* One-dimensional phase mapping calculations are only possible
     with version 3.x and later of ChemApp. */
  printf("Performing one-dimensional phase mapping calculations\n");

  /* Initialise the variable to zero that holds the number of
     times tqmap found results */
  resno = 0;
	   
  /* Remove all previous conditions */
  tqremc(-2, &noerr);
  if (noerr) abortprog(__LINE__,"tqremc",noerr);

  /* Determine the index number for the phase SiO2(quartz) */
  tqinp("SiO2(quartz)", &lint, &noerr);
  if (noerr) abortprog(__LINE__,"tqinp",noerr);

  /* Enter one mol of SiO2 */
  tqsetc("IA", lint, 0, 1.0, &numcon, &noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);

  /* The temperature search interval is supposed to range from 300 to
     3000 K: */
  darray2[0] = 300.0;
  darray2[1] = 3000.0;
    
  /* First call to tqmap, note the "f" ("first") in the option
     parameter */
  tqmap("tf", 0, 0, darray2, &icont, &noerr);
  if (noerr) abortprog(__LINE__,"tqmap",noerr);

  /* The variable resno keeps track of the number of times we call
     tqmap: */
  resno++;

  /* Retrieve and print the temperature, which we know is darray2[0]: */
  tqgetr("t", 0, 0, &d1, &noerr);
  if (noerr) abortprog(__LINE__,"tqgetr",noerr);
  printf("*** Lower interval boundary: %g K\n", d1);

  /* For as long as icont is positive, we need to make further calls
     to tqmap */
  while (icont) {

    /* tqmap is called again. Note the "n" ("next") in the option
       parameter. If we are at the first phase boundary (resno is
       still 2), we call tqmapl for a change to produce a ChemSage
       output table... */
    if (resno == 2) {
      printf("*** ChemSage result table "
	     "for the first phase boundary found:\n");
      fflush(NULL);
      tqmapl("tn", 0, 0, darray2, &icont, &noerr);
      fflush(NULL);
      if (noerr) abortprog(__LINE__,"tqmapl",noerr);
      printf("\n");
	  
      /* ...otherwise we just call tqmap: */
    } else {
      tqmap("tn", 0, 0, darray2, &icont, &noerr);
      if (noerr) abortprog(__LINE__,"tqmap",noerr);
    }
    resno++;

    /* Get the temperature... */
    tqgetr("t", 0, 0, &d1, &noerr);
    if (noerr) abortprog(__LINE__,"tqgetr",noerr);

    /* ...and print the entry for the result table. If we have called
       tqmap twice already, we know that we found a phase boundary. If
       not, we have retrieved the temperature value of the upper
       interval boundary (darray2[1]): */
	
    if (resno > 2) {
      printf("*** Phase boundary found at %g K\n", d1);
    } else {
      printf("*** Upper interval boundary: %g K\n", d1);
    }
  }


  /* With the above example the temperatures of all phase
     boundaries in a system which contains 1 mol of SiO2 have been
     calculated. Thus the phase boundaries determined reflect the
     stability ranges of the various modifications of SiO2. Also
     note that the first two temperatures determined are _no_
     phase boundaries, but the lower and upper limit of the search
     interval (darray2[0] and darray2[1]). */

  }

  /* Since from now on we will be using streams instead of global
     conditions, we first remove all conditions and targets set up
     to this point.  Note that using a "-1" instead of the "-2" in
     the call to tqremc would also reset ChemApp to default units
     and values.*/
  tqremc(-2, &noerr); 
  if (noerr) abortprog(__LINE__,"tqremc",noerr);


  printf("\n\nCalculations using streams:\n\n");

  /* Set name and temperature for a stream via the array tp, which
     will be passed to tqsttp when each of the streams is created. */
  tp[0] = 1000.0;
  tp[1] = 1.0;

  /* Create 3 streams: */
  tqsttp("stream1", tp, &noerr);
  if (noerr) abortprog(__LINE__,"tqsttp",noerr);
  tqsttp("stream2", tp, &noerr);
  if (noerr) abortprog(__LINE__,"tqsttp",noerr);
  tqsttp("stream3", tp, &noerr);
  if (noerr) abortprog(__LINE__,"tqsttp",noerr);

  /* Set constituent amounts for each stream */
  tqstca("stream1", 1, 4, 1.0,&noerr); 
  if (noerr) abortprog(__LINE__,"tqstca",noerr);
  tqstca("stream2", 1, 12, 3.0,&noerr);
  if (noerr) abortprog(__LINE__,"tqstca",noerr);
  tqstca("stream3", 1, 8, 2.0,&noerr);
  if (noerr) abortprog(__LINE__,"tqstca",noerr);
    
  /* Remove the last stream */
  tqstrm("stream3", &noerr);
  if (noerr) abortprog(__LINE__,"tqstrm",noerr);
    
  /* Set equilibrium temperature */
  tqstec("t ", 0, 1800.0, &noerr);
  if (noerr) abortprog(__LINE__,"tqstec",noerr);

  /* Calculate and list equilibrium */                   
  printf("\n\n**** Begin output table produced by tqcel\n");
  fflush(NULL);
  tqcel(" ",0,0,darray2,&noerr); 
  fflush(NULL);
  printf("\n**** End output table produced by tqcel\n\n\n");

  if (noerr) abortprog(__LINE__,"tqcel",noerr);

  /* Get thermodynamic properties of a stream (note that as of
     version 3.2.0 of ChemApp, tqstxp can also be called _before_
     the equilibrium calculation) */
  tqstxp("stream1", "H", &d1, &noerr);
  if (noerr) abortprog(__LINE__,"tqstxp",noerr);

  printf("Enthalpy of \"stream1\": %g\n\n", d1);



  printf("\nDemonstrating the use of ChemApp "
	 "subroutines involving sublattices.\n\n");
  /* To demonstrate the subroutines dealing with sublattices, a
     different data-file (subl-ex.dat) needs to be loaded. This
     data-file contains an extract of the system Co-Cr-Fe: the phase
     SIGMA:30, which is modelled according to the sublattice
     formalism, and the BCC phase, described by a Redlich-Kister
     polynomial. Both phases are each included twice, to account for
     miscibility gaps. */

  tqopna("subl-ex.dat", unitno, &noerr);
  if (noerr) abortprog(__LINE__,"tqopna",noerr);

  /* Read data-file */
  tqrfil(&noerr);
  if (noerr) abortprog(__LINE__,"tqrfil",noerr);

  /* Close data-file */
  tqclos(unitno, &noerr);
  if (noerr) abortprog(__LINE__,"tqclos",noerr);
      
  /* The first of the two identical copies of the SIGMA:30 phase,
     which ChemApp calls SIGMA:30#1, will be investigated with
     respect to the number of sublattices, the number of sublattice
     constituents on each sublattice, and the names of the
     sublattice constituents. */

  /* Get the index number for the phase SIGMA:30#1 */
  strcpy(pname, "SIGMA:30#1");
  tqinp(pname, &indexp, &noerr);
  if (noerr) abortprog(__LINE__,"tqinp",noerr);

  /* Get the number of sublattices */
  tqnosl(indexp, &nsl, &noerr);
  if (noerr) abortprog(__LINE__,"tqnosl",noerr);
  printf("%s has %li sublattices\n",pname, nsl);

            
  /* Loop over all sublattices */
  for (indexl = 1; indexl<=nsl; indexl++) {

    /* Get the number of sublattice constituents */
    tqnolc(indexp, indexl, &nslcon, &noerr);
    if (noerr) abortprog(__LINE__,"tqnolc",noerr);
    printf("Sublattice %li has %li constituents with the following names:\n",
	   indexl, nslcon);

    /* Get the name of each sublattice constituent */
    for (indexc = 1; indexc<=nslcon; indexc++) {
      tqgnlc(indexp, indexl ,indexc, cname, &noerr);
      if (noerr) abortprog(__LINE__,"tqgnlc",noerr);

      /* The reverse (getting the index number for the name of the
	 sublattice constituent just retrieved), is rather
	 superfluous here, and only used to demonstrate the call to
	 tqinlc: */
      tqinlc(cname, indexp, indexl, &lint, &noerr);
      if (noerr) abortprog(__LINE__,"tqinlc",noerr);
      printf("   %li: %s\n", lint, cname);
    }
  }

  

  /* Set the temperature to 1000 K */
  tqsetc("T", 0, 0, 1000.0, &numcon, &noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);

  /* Set the incoming amounts to 0.25 mol Co, 0.25 mol Cr, 
     and 0.5 mol Fe */
  tqinsc("Co", &indexc, &noerr);
  if (noerr) abortprog(__LINE__,"tqinsc",noerr);
  tqsetc("ia", 0, indexc, 0.25, &numcon, &noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);

  tqinsc("Cr", &indexc, &noerr);
  if (noerr) abortprog(__LINE__,"tqinsc",noerr);
  tqsetc("ia", 0, indexc, 0.25, &numcon, &noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);
    
  tqinsc("Fe", &indexc, &noerr);
  if (noerr) abortprog(__LINE__,"tqinsc",noerr);
  tqsetc("ia", 0, indexc, 0.50, &numcon, &noerr);
  if (noerr) abortprog(__LINE__,"tqsetc",noerr);
    
  printf("\nCalculate the equilibrium, "
	 "get information on the stable sublattice phases.\n\n");

  /* Calculate the equilibrium */
  darray2[0]=0.0;
  tqce(" ", 0, 0, darray2, &noerr);
  if (noerr) abortprog(__LINE__,"tqce",noerr);
    
  /* Get information on the stable sublattice phases. */

  /* Loop over all phases, and if a phase is stable and a sublattice
     phase, print information about the sublattices */
  tqnop(&nphase, &noerr);
  if (noerr) abortprog(__LINE__,"tqnop",noerr);
  for (indexp = 1; indexp<=nphase; indexp++) {

    /* Check if the phase is stable, otherwise we don't need to
       consider it */
    tqgetr("a", indexp, 0, &d1, &noerr);
    if (noerr) abortprog(__LINE__,"tqgetr",noerr);
    if (d1 > 0) {
	
      /* Get the name and model of the phase */
      tqgnp(indexp, pname, &noerr);
      if (noerr) abortprog(__LINE__,"tqgnp",noerr);
      tqmodl(indexp, mname, &noerr);
      if (noerr) abortprog(__LINE__,"tqmodl",noerr);

      /* If the model name of the phase starts with 'SUB' we have a
	 sublattice phase */
      if (strncmp(mname, "SUB", 3) == 0) {

	/* Print a header similar to the one in the ChemSage output
	   table */
	printf("Mole fraction of the sublattice constituents in %s\n",
	       pname);
    
	/* Get the number of sublattices */
	tqnosl(indexp, &nsl, &noerr);
	if (noerr) abortprog(__LINE__,"tqnosl",noerr);
	  
	/* Loop over all sublattices */
	for (indexl=1; indexl<=nsl; indexl++) {
	    
	  printf("Sublattice %li\n", indexl);
	  /* Get the number of sublattice constituents */
	  tqnolc(indexp, indexl, &nslcon, &noerr);
	  if (noerr) abortprog(__LINE__,"tqnolc",noerr);
	    
	  for (indexc=1; indexc<=nslcon; indexc++) {
	    /* Get the name of each sublattice constituent... */
	    tqgnlc(indexp, indexl, indexc, cname, &noerr);
	    if (noerr) abortprog(__LINE__,"tqgnlc",noerr);
	      
	    /* ... and its mole fraction in the sublattice */
	    tqgtlc(indexp, indexl, indexc, &d1, &noerr);
	    if (noerr) abortprog(__LINE__,"tqgtlc",noerr);
	      
	    printf("%-25s %f\n", cname, d1);
	  }
	}
      }
    }
  }
  printf("\n");

   

  /* The following section demonstrates the use of subroutines that
     were introduced in ChemApp V4.0.0 to support "transparent
     data-files" */
  
  /* Retrieve the licensee's user ID */
  tqgtid(id, &noerr);
  if (noerr) abortprog(__LINE__,"tqgtid",noerr);

  /* Retrieve the licensee's name */
  tqgtnm(name, &noerr);
  if (noerr) abortprog(__LINE__,"tqgtnm",noerr);

  /* Retrieve the program ID */
  tqgtpi(pid, &noerr);
  if (noerr) abortprog(__LINE__,"tqgtpi",noerr);

  /* Print all three */
  printf("Licensee's user ID: %s\n", id);
  printf("Licensee's name   : %s\n", name);
  printf("Program ID        : %s\n\n", pid);


  /* The following pieces of information are only meaningful if a
      version of ChemApp is used that requires a dongle (hardware
      key).  Get the HASP dongle type and id */
  tqgthi(haspt, &haspid, &noerr);
  if (noerr) abortprog(__LINE__,"tqgthi",noerr);

  /* Get the ChemApp license expiration date (month and year) */
  tqgted(&edmon, &edyear, &noerr);
  if (noerr) abortprog(__LINE__,"tqgted",noerr);

  /* Print info if HASP dongle is used: */
  if (haspid) {
    printf("HASP dongle type : %s\n", haspt);
    printf("HASP dongle id   : %li\n", haspid);
    printf("ChemApp license expiration date (month/year): %li/%li\n", 
	   edmon, edyear);
  } else {
    printf("This ChemApp version does not require a "
	   "HASP hardware key (dongle)\n");
  }  



  /* Get the default unit number associated with error output */
  tqgio("ERROR", &errorunit, &noerr);
  if (noerr) abortprog(__LINE__,"tqgio",noerr);

  /* Turn off automatic error reporting for the next call to tqcio. */
  tqcio("ERROR", 0, &noerr);
  if (noerr) abortprog(__LINE__,"tqcio",noerr);
  
  /* The following section is only executed if the file "cosiex.cst",
     a sample thermochemical data-file in transparent format, is
     present. It is not included in regular distributions of ChemApp */

  tqopnt("cosiex.cst",unitno,&lint);

  /* Turn automatic error reporting back on. */
  tqcio("ERROR", errorunit, &noerr);
  if (noerr) abortprog(__LINE__,"tqcio",noerr);

  if (lint) {
    printf("Skipping transparent file subroutines, since file "
	   "'cosiex.cst' cannot be read\n");
  } else {
    
    /* Read data-file */
    tqrcst(&noerr);
    if (noerr) abortprog(__LINE__,"tqrcst",noerr);
    
    /* Close data-file */
    /* Again, the routine for closing the data-file is not a standard
       C library routine like fclose, but a special one to make sure the
       file that was previously opened under the unit number specified
       is closed. Both tqopnb and tqclos are not actually a part of ChemApp
       itself, but of ChemApp's C interface. */
    tqclos(unitno,&noerr);
    if (noerr) abortprog(__LINE__,"tqclos",noerr);
   
  /* Once the transparent data-file has been read, information on its 
     header can be retrieved.*/
    tqgtrh(&tfhver, tfhnwp, tfhvnw, tfhnrp, tfhvnr, tfhdtc, tfhdte, 
	   tfhid, tfhusr, tfhrem, &noerr);
    if (noerr) abortprog(__LINE__,"tqgtrh",noerr);

    printf("Version number of the transparent file header format: %li\n", 
	   tfhver);
    printf("Name of the program which wrote the data-file: %s\n", tfhnwp);
    printf("Version number of the writing program: %li.%li.%li\n", 
	   tfhvnw[0],tfhvnw[1],tfhvnw[2]);
    printf("Programs which are permitted to read the data-file: %s\n", tfhnrp);
    printf("Min. version number of the reading program: %li.%li.%li\n", 
	   tfhvnr[0],tfhvnr[1],tfhvnr[2]);
    printf("File was created on %li/%02li/%02li %02li:%02li:%02li \n", 
	   tfhdtc[0],tfhdtc[1],tfhdtc[2],tfhdtc[3],tfhdtc[4],tfhdtc[5]);
    printf("File will expire on %li/%02li/%02li %02li:%02li:%02li \n", 
	   tfhdte[0],tfhdte[1],tfhdte[2],tfhdte[3],tfhdte[4],tfhdte[5]);
    printf("Licensee's user ID(s): %s\n", tfhid);
    printf("Licensee's name: %s\n", tfhusr);
    printf("Remarks : %s\n", tfhrem);

  }


  printf("\nEnd of output from cademo1.\n");


  return 0;
}

