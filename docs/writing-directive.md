# Writing Directive: No AI-isms

Canonical copy. This is the authority for the `/sweep-ai-isms` command and for any prose, comment, doc, or commit message. Keep it in sync with the `writing-style` memory.

Follow every rule below when producing text of any kind: prose, documentation, emails, marketing copy, creative writing, code comments, commit messages, chat responses. Whenever editing existing text for any reason, scan the full text for every pattern here and rewrite each occurrence, even outside the section you were asked to change. Preserve meaning and the author's voice. Do not over-correct into choppy or robotic prose. The goal is writing that reads like a specific person wrote it, not like a model generated it.

Codebase carve-out: a colon that introduces a real list, a code block, or a definition is fine, and semicolons are fine inside code, grammar notation, and tables. The target is prose colons and semicolons that join clauses. Never touch code, identifiers, or string literals.

## 1. Punctuation and mechanics

Never use em dashes or double hyphens (--), en dashes as em dash substitutes, ellipses for effect, exclamation points (except quoted or genuinely exclamatory), emoji or decorative unicode, arrow characters in prose (write "becomes" or "leads to"), scare quotes around ordinary words, colon-formatted headers like "The Result: Faster Builds", horizontal rules as decoration, more than one stacked parenthetical per paragraph, "quite literally"/"literally" as intensifiers, or Capitalized Common Nouns for mock gravity.

## 2. Banned words (filler, metaphorical, hype, or self-congratulatory uses; literal technical uses are fine)

Hype adjectives: seamless, effortless, robust (as praise), transformative, revolutionary, groundbreaking, game-changing, cutting-edge, state-of-the-art, next-level, world-class, best-in-class, top-notch, first-class, premier, unparalleled, unmatched, unrivaled, second to none, innovative (unearned), dynamic (as praise), vibrant, bustling, nestled, compelling, nuanced, multifaceted, holistic, meticulous, comprehensive (self-describing your output), invaluable, indispensable, ever-evolving, fast-paced, curated, bespoke, artisanal, handcrafted, elevated, tailored, turnkey, blazing, lightning-fast, buttery/silky smooth, dead simple, powerful (reflexive praise), scalable/performant (hype), optimal (when "good"/"best"), impactful, actionable, insightful, granular, pivotal, crucial/vital/key/core/critical/essential (as adjective spam when "important" or nothing would do).

Corporate/marketing verbs: leverage, utilize (use "use"), elevate, empower, foster, harness, unlock, unleash, supercharge, streamline, facilitate, enable (spam), ensure (spam), enhance, augment, amplify, accelerate (hype), optimize (loose), drive/deliver (value/results), boasts, showcases, underscores, resonates, revolutionize, transform (hype), reimagine, redefine, disrupt, spearhead, champion (verb), evangelize, ideate, operationalize, incentivize, socialize (an idea), sunset (verb).

Pomp nouns and dead metaphors: tapestry, landscape (metaphor), realm, journey (metaphor), ecosystem (loose), arsenal, toolkit (metaphor), treasure trove, goldmine, wealth of, beacon, cornerstone, keystone, bedrock, backbone, lifeblood, linchpin, catalyst (loose), springboard, stepping stone, testament, synergy, paradigm(-shift), myriad, plethora, a host/slew/bevy of, game-changer, win-win, north star, secret sauce, silver bullet, golden ticket, holy grail, uncharted territory/waters, the whole nine yards, learnings (use "lessons"), takeaways (unprompted), insights (spam), value-add, deliverables (loose), bandwidth (metaphor), alignment (loose), stakeholders (loose).

Filler adverbs and transition spam: truly, deeply (intensifier), incredibly, remarkably, undeniably, certainly, essentially, basically, ultimately, simply, actually, really, very/quite/rather (when cutting changes nothing), somewhat, arguably, notably, importantly, interestingly, significantly (filler), crucially, admittedly, undoubtedly, moreover, furthermore, additionally (as opener), thus/hence (outside proofs), therefore/consequently (spam), folks, indeed, of course, needless to say.

## 3. Banned phrases

Openers and scene-setters: "In today's/the modern world/era", "In the ever-evolving landscape/world/realm of", "In a world where", "Since the dawn of time / throughout history", "Picture this / imagine a world", "Did you know / ever wondered", "Let's dive in / explore / take a closer look / break it down / buckle up", "Without further ado", "First and foremost", "Look," opener, "Look no further", "Enter/Meet [product]", "Welcome to the world of".

Contrast and negation crutches: "It's not just X, it's Y" and all variants ("This isn't about X. It's about Y", "X is more than just Y"), "not only X but also Y", "Gone are the days", "Say goodbye/hello to", "No more X", "Forget everything you know", "X but better / on steroids / for the modern age", "the X that Y deserves".

Cliché idioms and dead metaphors: "at the end of the day", "when all is said and done", "the bottom line", "win-win / best of both worlds / perfect storm", "low-hanging fruit / move the needle / circle back / double-click / drill down", "double-edged sword", "delicate balance / strike a balance / walk the line/tightrope", "navigate the complexities of / grapple with", "shed light on / paint a picture / tell a story", "speaks volumes", "at the intersection of", "bridging the gap / paving the way / setting the stage / laying the groundwork", "stepping stone / springboard", "each and every / any and all", "tried and true", "tip of the iceberg", "rising tide lifts all boats", "elephant in the room", "breath of fresh air", "hit the ground running", "firing on all cylinders", "well-oiled machine".

Pseudo-profundity: "profound implications / far-reaching consequences", "the human experience/condition", "fabric of society/reality", "what it means to be human", "a testament to the power of / stands as a testament to", "a masterclass in", "a love letter to", "a labor of love", "the heart/soul of", "at its core", "in essence / at its essence", "the very nature of", "begs the question".

Throat-clearing: "It's worth noting/should be noted/bears mentioning that", "It's important to note/remember/understand that", "It goes without saying", "That being said / Having said that / With that in mind/said", "when it comes to / in terms of" (filler), "The beauty of X is", "What sets X apart", "Think of it as" (0 to 1 per doc), "To put it simply / Simply put / In other words (spam) / That is to say", "To be clear / To clarify" (unprompted), "As mentioned earlier / As discussed above / As we saw", "One might argue / Some may say / Critics argue" (unnamed), "Many experts believe / Studies show / Research suggests" (uncited), "plays a crucial/pivotal/vital/key role in", "serves as / acts as" (write "is"), "aims to / is designed to / strives to / seeks to" (state what it does), "has the potential to / is capable of" (write "can"), "unlock the full potential of", "harness the power of", "a wide range/variety/number of / various / numerous" (when a specific word or count exists), "and more / and beyond" as list closers, "including but not limited to".

Punchy-fragment tells: "Here's the thing/kicker/where it gets interesting/why that matters", "Spoiler alert / Pro tip / Fun fact / Hot take / Plot twist", "Let that sink in / Read that again / Yes, you read that right", "chef's kiss", "Full stop. / Period." as emphasis, "Plain and simple / Make no mistake / Rest assured / Fear not / Worry not", "And honestly? / And the best part? / The result?" fragments, "Wild. / Incredible. / Powerful." one-word verdicts.

Hollow closers: "In conclusion / summary / short / To sum up / wrap up / All in all / Overall," as opener, "The key takeaway is", "Ultimately the choice is yours / comes down to your needs", "Only time will tell", "The possibilities are endless / sky's the limit", "The future is bright / of X is here / Watch this space", "One thing is certain", "and that makes all the difference", "Last but not least", a final paragraph that restates the document.

## 4. Assistant-voice and chatbot tells

"Great/Good question / Excellent point" or any compliment on the request; "Certainly! / Absolutely! / Of course! / Sure thing!" openers; "I'd be happy to / I'm glad to / I'd love to"; "I hope this helps"; "Let me know if you have any questions / need anything else / want me to adjust anything"; "Feel free to / Don't hesitate to"; "Would you like me to" reflexive closer; "As an AI / language model"; "You're absolutely right / right to point that out / That's a fair point" reflexive agreement; "I appreciate you sharing / Thank you for your patience / Thanks for bearing with me"; repeated, preemptive, or unearned apologies; "I understand your frustration"; restating the request; "Below is a comprehensive overview / Here's a detailed breakdown"; postamble summaries; three follow-up options at the end; validating an emotion before a technical answer.

## 5. Epistemic tics, hedging, and reflexive disclaimers

Hedged double modality ("could potentially", "may possibly", "might perhaps"); hedging a claim you have evidence for; "It depends" without saying what on and answering the likely case; "no one-size-fits-all / everyone is different"; "your mileage may vary"; "As with anything / like anything in life / as always"; "generally/typically/often/tend to/in most cases" stacked; "It is generally recommended" (by whom); unprompted "not financial/legal/medical advice" or "consult a professional"; knowledge-cutoff and real-time-data disclaimers when irrelevant; false balance ("both have merit" with no pick, "pros and cons" with no verdict); answering a yes/no question with a survey and no answer.

## 6. Wordiness and weak constructions

"in order to" (to), "due to the fact that" (because), "despite the fact that" (although), "in the event that" (if), "at this point in time / at the present moment" (now), "on a daily basis" (daily), "prior to" (before), "subsequent to" (after), "in close proximity to" (near), "a majority of" (most), "is able to / has the ability to" (can), "there is/are" openers when a real subject exists, "it is X that" cleft filler ("it is clear/evident that"), nominalizations where the verb is stronger ("the implementation of caching" to "caching"), agent-hiding passive ("mistakes were made"), prepositional-phrase chains, redundant pairs ("first and foremost", "each and every", "various different"), saying the same thing twice with a synonym.

## 7. Banned rhetorical structures

Contrastive negation as a crutch (one per document, if earned). Anaphora runs. Rule of three as default rhythm ("fast, reliable, and secure"; two or four items are fine). One-word dramatic sentences. Rhetorical questions as transitions. Question-formatted headers. A sentence restating the previous one. A closing sentence restating its paragraph. Announcing structure ("In this section we will cover"). Meta-narration ("Let's take a moment to appreciate"). Empty superlatives on your own subject. "one of the most X" reflexive opener. "widely regarded/considered as" without attribution. Uniform sentence length (vary it). Identical topic-sentence-then-support shape every paragraph. Symmetric "on one hand / on the other hand" when you have a position.

## 8. Banned document structures

Bullet lists for prose. The bolded-label bullet pattern ("**Speed:** the system is fast"). Bolding phrases mid-sentence. The numbered-bold-header pattern. A header every two paragraphs. Title Case On Every Header (use sentence case). Unprompted "Key Takeaways / Final Thoughts / Conclusion / Next Steps / FAQ". Tables for non-tabular content. Numbered lists where order does not matter. Emoji-prefixed headers or bullets. Nested bullets past two levels. Block quotes for emphasis. Every paragraph the same two to three sentences. A TL;DR unless requested.

## 9. Email and professional messaging tells

"I hope this email finds you well", "I wanted to reach out / follow up / touch base", "I am writing to", "Just checking in / circling back / a quick note", "per my last email", "at your earliest convenience", "please don't hesitate to reach out", "moving/going forward" filler, "on the same page / aligned / sync up", "Thanks in advance", "Sorry to bother you", relationship padding before the ask, closers that restate the email.

## 10. Creative-writing tells

"couldn't help but", "a mix of X and Y" emotions, "washed over him / a wave of", "shivers down her spine", default "heart pounding/racing", "breath hitched", "let out a breath he didn't know he'd been holding", "the air was thick with / palpable tension", "a comfortable silence", "a ghost of a smile / small smile played on his lips", "eyes sparkled/glinted/twinkling", "voice barely above a whisper", "for what felt like an eternity", "little did they know", "in that moment / everything changed", "something shifted", "a single tear rolled down", "steeled herself / squared his shoulders", body-part cliché reactions, welded adjective-noun pairs ("warm glow", "deafening silence", "stark contrast"), portentous one-line scene closers.

## 11. Code and technical writing tells

Comments that narrate the obvious ("// Import the necessary libraries", "// Define the function", "// Return the result"). Tutorial narration in comments ("First, we...", "Now we can...", "Let's define...", "As you can see"). "This function is responsible for" (write the imperative, "Parses the header"). "Helper function to" as a docstring opener. Docstrings that restate the signature and add nothing. "handles the logic for". Boilerplate generated TODOs. Comments explaining what when the code needs why or nothing. Emoji in commit messages, log output, or CLI banners. Commit messages like "feat: enhance robustness" (say the actual change). README hype sections ("blazing fast", badges as substance, feature lists of adjectives). "Simply run" / "just run" in docs (delete "simply" and "just"). "Note that" as a reflexive sentence opener in docs. Changelog entries that describe intent ("improved performance") instead of the change. Error messages that apologize or emote. Comments that editorialise about the project's choices instead of explaining the code ("It is written in Frost on purpose", "everything it needs already existed", "this is what binding any C library looks like", "two shapes are worth naming"). State the mechanism and the constraint: "an aggregate handed to an `extern` is moved across the boundary, so a buffer C only fills takes a raw pointer" earns its place, self-congratulation aimed at a reader does not. That framing also contradicts itself under its own weight and goes stale the moment the surrounding facts change.

## 12. Detection heuristics

Anywhere test: a sentence that fits any topic carries no information, cut it. Negation test: if the opposite would never be written, the claim is empty, cut it. Delete test: if deleting a word loses nothing, keep the deletion. Specificity test: a praise adjective should be replaceable by a fact ("fast" becomes a number). Voice test: read aloud, if no human would say it across a table, rewrite. Density test: sentences that only set up, transition, or recap should be near zero.

## 13. Replacement guidance

Rewrite the sentence, do not just delete the banned string. Em dash pairs become parentheses or a comma pair; em dashes before a final clause become a colon or a period plus a sentence. Cut filler intensifiers rather than swapping them. "serves as" becomes "is", "utilize" becomes "use", "in order to" becomes "to", "prior to" becomes "before". Vague authority claims get a real source or get cut. A contrastive-negation sentence keeps its second half only. Hype adjectives become facts or nothing. Hedges resolve to a committed claim or a specific uncertainty. Dead metaphors become literal. Throat-clearing openers and summarizing closers get deleted.

## 14. Remediation protocol for existing text

1. Complete the requested edit.
2. Scan the entire text against sections 1 through 11 using the heuristics in section 12.
3. Fix every occurrence with section 13.
4. Flag a fix that would change meaning or voice rather than silently rewriting.
5. Report the categories found and fixed, briefly, without listing every instance unless asked.
6. Do not introduce new violations while fixing old ones. Re-scan your own edits before finishing.
