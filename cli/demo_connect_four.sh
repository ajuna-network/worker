#!/bin/bash

# setup:
# build ajuna node with skip-ias-check on branch "validateer-setup"
#   cargo build --release --features skip-ias-check
#
# run ajuna node
#   ./target/release/ajuna  --dev --tmp --ws-port <NODEPORT>
#
# run worker inside the bin folder:
#   rm light_client_db.bin
#   rm -r shards
#   rm -r sidechain_db
#   export RUST_LOG=integritee_service=info,ita_stf=debug
#   ./integritee-service init-shard
#   ./integritee-service shielding-key
#   ./integritee-service signing-key
#   ./integritee-service -P <WORKERPORT> -p <NODEPORT> -r <REMOTE-ATTESTATION-PORT> run --dev --skip-ra
#
# then run this script

# usage:
#  export RUST_LOG=integritee-cli=info,ita_stf=info
#  demo_connect_four.sh -p <NODEPORT> -A <WORKER1PORT> -B <WORKER2PORT> -m file
#
# if -m file is set, the mrenclave will be read from file  ~/mrenclave.b58


while getopts ":m:p:P:" opt; do
    case $opt in
        m)
            READMRENCLAVE=$OPTARG
            ;;
        p)
            NPORT=$OPTARG
            ;;
        P)
            RPORT=$OPTARG
            ;;
    esac
done

# using default port if none given as arguments
NPORT=${NPORT:-9944}
RPORT=${RPORT:-2000}

echo "Using node-port ${NPORT}"
echo "Using trusted-worker-port ${RPORT}"

BALANCE=1000

CLIENT="./../bin/integritee-cli -p ${NPORT} -P ${RPORT}"

if [ "$READMRENCLAVE" = "file" ]
then
    read MRENCLAVE <<< $(cat ~/mrenclave.b58)
    echo "Reading MRENCLAVE from file: ${MRENCLAVE}"
else
    # this will always take the first MRENCLAVE found in the registry !!
    read MRENCLAVE <<< $($CLIENT list-workers | awk '/  MRENCLAVE: / { print $2; exit }')
    echo "Reading MRENCLAVE from worker list: ${MRENCLAVE}"
fi
[[ -z $MRENCLAVE ]] && { echo "MRENCLAVE is empty. cannot continue" ; exit 1; }

echo ""
echo "* Create account for Alice"
ACCOUNTALICE=//Alice
echo "  Alice's account = ${ACCOUNTALICE}"
echo ""

echo "* Create account for Bob"
ACCOUNTBOB=//Bob
echo "  Bob's account = ${ACCOUNTBOB}"
echo ""

echo "* Issue ${BALANCE} tokens to Alice's account"
${CLIENT} trusted set-balance ${ACCOUNTALICE} ${BALANCE} --mrenclave=${MRENCLAVE} --direct
echo ""
sleep 1

echo "* Issue ${BALANCE} tokens to Bob's account"
${CLIENT} trusted set-balance ${ACCOUNTBOB} ${BALANCE} --mrenclave=${MRENCLAVE} --direct
echo ""
sleep 1

echo "Queue Game for Alice (Player 1)"
${CLIENT} queue-game ${ACCOUNTALICE}
echo ""
sleep 1

echo "Queue Game for Bob (Player 2)"
${CLIENT} queue-game ${ACCOUNTBOB}
echo ""
sleep 1

echo "waiting"
sleep 45

echo "Turn for Alice (Player 1)"
${CLIENT} trusted play-turn ${ACCOUNTALICE} 3 --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1

echo "Turn for Bob (Player 2)"
${CLIENT} trusted play-turn ${ACCOUNTBOB} 4 --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1

echo "Turn for Alice (Player 1)"
${CLIENT} trusted play-turn ${ACCOUNTALICE} 2 --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1

echo "Turn for Bob (Player 2)"
${CLIENT} trusted play-turn ${ACCOUNTBOB} 3 --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1

echo "waiting"
sleep 5

echo "Board after 2 turns"
${CLIENT} trusted get-board ${ACCOUNTBOB} --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1


echo "Turn for Alice (Player 1)"
${CLIENT} trusted play-turn ${ACCOUNTALICE} 2 --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1

echo "Turn for Bob (Player 2)"
${CLIENT} trusted play-turn ${ACCOUNTBOB} 5 --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1

echo "Turn for Alice (Player 1)"
${CLIENT} trusted play-turn ${ACCOUNTALICE} 2 --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1

echo "Turn for Bob (Player 2)"
${CLIENT} trusted play-turn ${ACCOUNTBOB} 1 --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1

echo "waiting"
sleep 5

echo "Board after 4 turns"
${CLIENT} trusted get-board ${ACCOUNTBOB} --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1

echo "Turn for Alice  (Player 1)"
${CLIENT} trusted play-turn ${ACCOUNTALICE} 2 --direct --mrenclave=${MRENCLAVE}
echo ""
sleep 1

echo "waiting"
sleep 5

echo "Board after end of game"
${CLIENT} trusted get-board ${ACCOUNTBOB} --direct --mrenclave=${MRENCLAVE}
echo ""
