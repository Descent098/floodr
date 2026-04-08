<script lang="ts">
    let rampup = $state(0)
    let iterations = $state(1)
    let averageRequestTime = $state(50) // in ms
    let concurrency = $state(1)

    let requestsPerIteration = $state(1)
    let specifiedDelay = $state(0) // ms
    const iterationStartupDelay = $derived(rampup/iterations)



    const approximateRuntimePerIteration = $derived(iterationStartupDelay + (((requestsPerIteration-1)*specifiedDelay)+(requestsPerIteration*averageRequestTime)))
    const totalRuntime = $derived((approximateRuntimePerIteration * iterations)/concurrency)

</script>

<div id="runtimeCalcResult">
    <h3>Runtime Calculator</h3>
    <form>
        <fieldset>
            <label for="rampup">Rampup</label>
            <input type="number" bind:value={rampup} min=0/>
        </fieldset>
        <fieldset>
            <label for="iterations">iterations</label>
            <input type="number" bind:value={iterations} min=0/>
        </fieldset>
        <fieldset>
            <label for="averageRequestTime">Average Request Time (ms)</label>
            <input type="number" bind:value={averageRequestTime}/>
        </fieldset>
        <fieldset>
            <label for="concurrency">Concurrency</label>
            <input type="number" bind:value={concurrency} min={concurrency}/>
        </fieldset>
        <fieldset>
            <label for="requestsPerIteration">requestsPerIteration</label>
            <input type="number" bind:value={requestsPerIteration} min=0/>
        </fieldset>
        <fieldset>
            <label for="specifiedDelay">Specified Delay</label>
            <input type="number" bind:value={specifiedDelay}/>
        </fieldset>
    </form>



<div>
    <h4>Total Runtime: {totalRuntime}ms (approximate)</h4>
    
</div>
</div>

<style>
    fieldset{
        border:none ;
        padding:.3rem;
        margin:0;
        label{
            display: block;
        }
        input{
            display: inline-block;
            max-width:50%;
        }
    }
    form{
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        grid-template-rows: repeat(3, 1fr);
        align-content: center;
        align-items: center;

        justify-items: start;
        justify-content: center;
    }
    @media screen and (max-width: 800px){
        fieldset{
            width: 100%;
        }
        form{
            grid-template-columns: 1fr;
            grid-template-rows: repeat(6, 1fr);

            
        }
        input{
            width:100%;
            max-width:100%;
        }

    }
</style>